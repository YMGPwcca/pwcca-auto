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
  power::{get_all_power_schemes, get_power_status, set_active_power_scheme},
  taskbar::taskbar,
};

use anyhow::Result;
use std::{mem::MaybeUninit, time::Duration};
use sysinfo::System;
use trayicon::{MenuBuilder, TrayIconBuilder};
use windows::Win32::{
  Foundation::WIN32_ERROR,
  UI::WindowsAndMessaging::{DispatchMessageW, GetMessageW, TranslateMessage},
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Events {
  LeftClickTrayIcon,
  Discord,
  Ethernet,
  TurnOffMonitor,
  RefreshRate,
  Exit,
}

static mut CONFIG: Config = Config::new();

fn setup_tray_icon_menu(tray_icon: &mut trayicon::TrayIcon<Events>) -> Result<()> {
  tray_icon
    .set_menu(
      &MenuBuilder::new()
        .checkable("Discord", unsafe { CONFIG.discord }, Events::Discord)
        .checkable("Ethernet", unsafe { CONFIG.ethernet }, Events::Ethernet)
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

fn main() -> Result<()> {
  // Check if another instance is running
  let system = System::new_all();
  if system.processes_by_name("PwccaAuto").count() > 1 {
    std::process::exit(0);
  }

  // Main application starts here
  unsafe {
    CONFIG = Config::read()?;
  };

  println!("Running Pwcca Auto");

  let (sender, receiver) = std::sync::mpsc::channel::<Events>();

  let mut tray_icon = TrayIconBuilder::new()
    .sender(move |e: &Events| sender.send(*e).unwrap())
    .icon_from_buffer(include_bytes!("../res/icon.ico"))
    .tooltip("Pwcca Auto")
    .on_click(Events::LeftClickTrayIcon)
    .build()
    .unwrap();

  setup_tray_icon_menu(&mut tray_icon)?;

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
      if bret.0 > 0 {
        let _ = TranslateMessage(msg.as_ptr());
        DispatchMessageW(msg.as_ptr());
      } else {
        break;
      }
    }
  }
  Ok(())
}

fn tray_thread(
  receiver: std::sync::mpsc::Receiver<Events>,
  mut tray_icon: trayicon::TrayIcon<Events>,
) {
  // Initialize the tray thread
  println!("  + Running Tray Thread");

  receiver.iter().for_each(|m| match m {
    Events::LeftClickTrayIcon => {
      tray_icon.show_menu().unwrap();
    }
    Events::Discord => {
      unsafe { CONFIG.toggle_discord() };
      let _ = setup_tray_icon_menu(&mut tray_icon);
    }
    Events::Ethernet => {
      unsafe { CONFIG.toggle_ethernet() };
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
  let discord_executable = String::from("Discord.exe");

  loop {
    if unsafe { CONFIG.discord } {
      // Get all output devices
      let all_outputs = enumerate_audio_devices(&DeviceType::Output)?;

      // Check if there are multiple output devices
      if all_outputs.len() > 1 {
        // Get the current default output device
        let current_output = get_default_device(&DeviceType::Output)?;

        // Check if Discord is running and recording from default input device
        let programs = get_active_audio_applications(&DeviceType::Input)?;

        if programs.contains(&discord_executable) {
          connected = true;

          // Switch to headphones if Discord is recording and speakers are the default
          if current_output.device_type == "Speakers" {
            let headphones = all_outputs
              .iter()
              .find(|device| device.device_type == "Headphones")
              .unwrap();

            change_default_output(headphones.device_id)?
          }
        } else if connected {
          connected = false;

          // Switch back to speakers if Discord is not recording and headphones are the default
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

fn connection_thread() {
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

  loop {
    let all_power_schemes = get_all_power_schemes()?;

    if on_battery_secs > unsafe { CONFIG.power.timer }
      || get_power_status().remaining_percentage < unsafe { CONFIG.power.percentage }
    {
      let power_scheme = all_power_schemes
        .iter()
        .find(|scheme| scheme.name == "POWERSAVER")
        .unwrap();
      let _ = set_active_power_scheme(&power_scheme.guid);
    } else {
      let power_scheme = all_power_schemes
        .iter()
        .find(|scheme| scheme.name == "Ultra")
        .unwrap();
      let _ = set_active_power_scheme(&power_scheme.guid);
    }

    if !get_power_status().is_plugged_in {
      on_battery_secs += 1;
    } else {
      on_battery_secs = 0;
    }

    std::thread::sleep(Duration::from_secs(1));
  }
}

fn taskbar_thread() {
  // Initialize the taskbar thread
  println!("  + Running Taskbar Thread");

  loop {
    taskbar();

    std::thread::sleep(Duration::from_millis(300));
  }
}
