#![allow(dead_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod mods;

use config::Config;
use mods::{
  connection::{is_ethernet_plugged_in, set_wifi_state},
  display::{get_all_frequencies, get_current_frequency, set_new_frequency, turn_off_monitor},
  media::{
    change_default_output, enumerate_audio_devices, get_active_audio_applications,
    get_default_device, init,
    types::{device::DeviceType, error::AudioDeviceError},
  },
  power::{
    get_active_power_scheme, get_all_power_schemes, get_power_status, set_active_power_scheme,
  },
  process::get_processes_by_name,
  startup::{
    registry::{get_all_startup_items, set_startup_item_state},
    task_scheduler::TaskScheduler,
  },
  taskbar::taskbar_automation,
};

use anyhow::Result;
use std::{mem::MaybeUninit, time::Duration};
use trayicon::{MenuBuilder, TrayIcon, TrayIconBuilder};
use windows::{
  core::w,
  Win32::{
    Foundation::{CloseHandle, HANDLE, HWND, TRUE, WIN32_ERROR},
    Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY},
    System::Threading::{GetCurrentProcess, OpenProcessToken},
    UI::WindowsAndMessaging::{
      DispatchMessageW, GetMessageW, MessageBoxW, TranslateMessage, MB_ICONERROR, MB_OK,
      MB_SYSTEMMODAL,
    },
  },
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Events {
  LeftClickTrayIcon,

  Startup,

  Discord,
  Ethernet,
  Taskbar,

  TurnOffMonitor,
  RefreshRate,

  Exit,
}

static mut CONFIG: Config = Config::new();

fn setup_tray_icon_menu(tray_icon: &mut trayicon::TrayIcon<Events>) -> Result<()> {
  tray_icon
    .set_menu(
      &MenuBuilder::new()
        .checkable(
          "Run with Windows",
          unsafe { CONFIG.startup },
          Events::Startup,
        )
        .separator()
        .checkable(
          "Microphone",
          unsafe { CONFIG.microphone.enabled },
          Events::Discord,
        )
        .checkable("Ethernet", unsafe { CONFIG.ethernet }, Events::Ethernet)
        .checkable(
          "Taskbar",
          unsafe { CONFIG.taskbar.enabled },
          Events::Taskbar,
        )
        .separator()
        .item("Turn off monitor", Events::TurnOffMonitor)
        .item(
          format!("Refresh Rate: {} Hz", get_current_frequency()).as_str(),
          Events::RefreshRate,
        )
        .separator()
        .item("Exit", Events::Exit),
    )
    .unwrap();

  unsafe {
    let _ = CONFIG.write();
  };

  Ok(())
}

fn is_elevated() -> Result<bool> {
  let mut elevated = false;
  let mut token_handle = HANDLE::default();

  unsafe {
    if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle).is_ok() {
      let mut elevation = TOKEN_ELEVATION::default();
      let size = std::mem::size_of_val(&elevation) as u32;
      let mut returnlength = 0;

      if GetTokenInformation(
        token_handle,
        TokenElevation,
        Some(&mut elevation as *mut _ as *mut _),
        size,
        &mut returnlength,
      )
      .is_ok()
      {
        elevated = elevation.TokenIsElevated != 0;
      }
    };

    let _ = CloseHandle(token_handle);
  };

  Ok(elevated)
}

fn main() -> Result<()> {
  // Check if another instance is running
  if get_processes_by_name("PwccaAuto")?.len() > 1 {
    unsafe {
      MessageBoxW(
        HWND::default(),
        w!("Another instance is already running"),
        w!("Error"),
        MB_SYSTEMMODAL | MB_ICONERROR | MB_OK,
      )
    };

    std::process::exit(1);
  }

  // Check if the process is elevated
  if !is_elevated()? {
    unsafe {
      MessageBoxW(
        HWND::default(),
        w!("This application requires administrator privileges"),
        w!("Error"),
        MB_SYSTEMMODAL | MB_ICONERROR | MB_OK,
      )
    };

    std::process::exit(1);
  }

  // Main application starts here
  unsafe {
    CONFIG = Config::read()?;
  };

  println!("Running Pwcca Auto");

  let (sender, receiver) = std::sync::mpsc::channel::<Events>();

  // Tray icon
  let mut tray_icon = TrayIconBuilder::new()
    .sender(move |e| sender.send(*e).unwrap())
    .icon_from_buffer(include_bytes!("../res/icon.ico"))
    .tooltip("Pwcca Auto")
    .on_click(Events::LeftClickTrayIcon)
    .build()
    .unwrap();

  setup_tray_icon_menu(&mut tray_icon)?;

  // Threading
  let _ = std::thread::Builder::new()
    .name("Startup_Thread".to_string())
    .spawn(startup_thread);
  let _ = std::thread::Builder::new()
    .name("Power_Thread".to_string())
    .spawn(power_thread);
  let _ = std::thread::Builder::new()
    .name("Media_Thread".to_string())
    .spawn(media_thread);
  let _ = std::thread::Builder::new()
    .name("Connection_Thread".to_string())
    .spawn(connection_thread);
  let _ = std::thread::Builder::new()
    .name("Taskbar_Thread".to_string())
    .spawn(taskbar_thread);
  let _ = std::thread::Builder::new()
    .name("Tray_Thread".to_string())
    .spawn(move || tray_thread(receiver, tray_icon));

  // Application loop
  loop {
    unsafe {
      let mut msg = MaybeUninit::uninit();
      let bret = GetMessageW(msg.as_mut_ptr(), None, 0, 0);
      if bret == TRUE {
        let _ = TranslateMessage(msg.as_ptr());
        DispatchMessageW(msg.as_ptr());
      } else {
        break;
      }
    }
  }

  Ok(())
}

fn startup_thread() -> Result<()> {
  // Initialize the tray thread
  println!("  + Running Startup Thread");

  loop {
    let disallow = ["Discord", "WallpaperEngine", "Overwolf", "Joplin.lnk"];
    let disallow: Vec<String> = disallow.iter().map(|s| s.to_string()).collect();

    let is_plugged_in = get_power_status().is_plugged_in;
    let startup_items = get_all_startup_items()?;

    for item in &startup_items {
      if disallow.contains(&item.name) {
        set_startup_item_state(item, is_plugged_in)
          .unwrap_or_else(|_| panic!("Cannot disable {} startup", item.name));
      }
    }

    std::thread::sleep(Duration::from_secs(1));
  }
}

fn tray_thread(receiver: std::sync::mpsc::Receiver<Events>, mut tray_icon: TrayIcon<Events>) {
  // Initialize the tray thread
  println!("  + Running Tray Thread");

  let task_scheduler = TaskScheduler::new().expect("Cannot construct task scheduler");

  receiver.iter().for_each(|m| match m {
    Events::LeftClickTrayIcon => {
      tray_icon.show_menu().unwrap();
    }
    Events::Startup => {
      unsafe { CONFIG.toggle_startup() };

      if unsafe { CONFIG.startup } {
        let _ = task_scheduler.create_startup_task("PwccaAuto");
      } else {
        let _ = task_scheduler.delete_startup_task("PwccaAuto");
      }

      let _ = setup_tray_icon_menu(&mut tray_icon);
    }
    Events::Discord => {
      unsafe { CONFIG.toggle_microphone() };
      let _ = setup_tray_icon_menu(&mut tray_icon);
    }
    Events::Ethernet => {
      unsafe { CONFIG.toggle_ethernet() };
      let _ = setup_tray_icon_menu(&mut tray_icon);
    }
    Events::Taskbar => {
      unsafe { CONFIG.toggle_taskbar() };
      let _ = setup_tray_icon_menu(&mut tray_icon);
    }
    Events::TurnOffMonitor => turn_off_monitor(),
    Events::RefreshRate => {
      let refresh_rate = get_current_frequency();
      let max_refresh_rate = get_all_frequencies().last().copied().unwrap();
      set_new_frequency(if refresh_rate == 60 {
        max_refresh_rate
      } else {
        60
      });

      let _ = setup_tray_icon_menu(&mut tray_icon);
    }
    Events::Exit => std::process::exit(0),
  });
}

fn media_thread() -> Result<(), AudioDeviceError> {
  // Initialize the media thread
  println!("  + Running Media Thread");

  init()?;

  let mut connected = false;

  loop {
    if unsafe { CONFIG.microphone.enabled } {
      let config_includes = unsafe { &CONFIG.microphone.apps };

      let all_outputs = enumerate_audio_devices(&DeviceType::Output)?;

      if all_outputs.len() > 1 {
        let current_output = get_default_device(&DeviceType::Output)?;

        let programs = get_active_audio_applications(&DeviceType::Input)?;

        if config_includes.iter().any(|e| programs.contains(e)) {
          connected = true;

          if current_output.device_type == "Speakers" {
            let headphones = all_outputs
              .iter()
              .find(|device| device.device_type == "Headphones")
              .unwrap();

            change_default_output(headphones.device_id)?
          }
        } else if connected {
          connected = false;

          if current_output.device_type == "Headphones" {
            let headphones = all_outputs
              .iter()
              .find(|device| device.device_type == "Speakers")
              .unwrap();

            change_default_output(headphones.device_id)?
          }
        }
      }
    }

    std::thread::sleep(Duration::from_secs(1));
  }
}

fn connection_thread() -> Result<()> {
  // Initialize the connection thread
  println!("  + Running Connection Thread");

  loop {
    if unsafe { CONFIG.ethernet } {
      let _ = set_wifi_state(!is_ethernet_plugged_in());
    }

    std::thread::sleep(Duration::from_secs(1));
  }
}

fn power_thread() -> Result<(), WIN32_ERROR> {
  // Initialize the power thread
  println!("  + Running Power Thread");

  let mut on_battery_secs = 0;
  let all_power_schemes = get_all_power_schemes()?;
  let power = unsafe { CONFIG.power };

  let powersaver = all_power_schemes
    .iter()
    .find(|scheme| scheme.name == "POWERSAVER")
    .unwrap();
  let ultra = all_power_schemes
    .iter()
    .find(|scheme| scheme.name == "Ultra")
    .unwrap();

  loop {
    if power.enabled {
      let is_plugged_in = get_power_status().is_plugged_in;

      if power.timer != 0 && on_battery_secs > power.timer {
        set_active_power_scheme(&powersaver.guid)?;
      }

      if power.percentage != 0
        && !is_plugged_in
        && get_power_status().remaining_percentage < power.percentage
      {
        set_active_power_scheme(&powersaver.guid)?;
      }

      if is_plugged_in && get_active_power_scheme()?.guid == powersaver.guid {
        set_active_power_scheme(&ultra.guid)?;
      }

      if !is_plugged_in {
        on_battery_secs += 1;
      } else {
        on_battery_secs = 0;
      }
    }

    std::thread::sleep(Duration::from_secs(1));
  }
}

fn taskbar_thread() {
  // Initialize the taskbar thread
  println!("  + Running Taskbar Thread");

  loop {
    if unsafe { CONFIG.taskbar.enabled } {
      taskbar_automation();
    }

    std::thread::sleep(Duration::from_secs(1));
  }
}
