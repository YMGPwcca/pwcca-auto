#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod mods;

use mods::{
  connection::{is_ethernet_plugged_in, set_wifi_state},
  display::{get_all_frequencies, get_current_frequency, set_new_frequency, turn_off_monitor},
  media::{
    change_default_output, enumerate_audio_devices, get_active_audio_applications, get_default_device, init,
    types::{device::DeviceType, error::AudioDeviceError},
  },
};
use std::{mem::MaybeUninit, time::Duration};
use sysinfo::System;
use trayicon::{MenuBuilder, TrayIconBuilder};
use windows::Win32::UI::WindowsAndMessaging::{DispatchMessageA, GetMessageA, TranslateMessage};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Events {
  RightClickTrayIcon,
  Discord,
  Ethernet,
  TurnOffMonitor,
  RefreshRate,
  Exit,
}

static mut DISCORD: bool = true;
static mut ETHERNET: bool = true;

fn setup_tray_icon_menu(tray_icon: &mut trayicon::TrayIcon<Events>) {
  tray_icon
    .set_menu(
      &MenuBuilder::new()
        .checkable("Discord", unsafe { DISCORD }, Events::Discord)
        .checkable("Ethernet", unsafe { ETHERNET }, Events::Ethernet)
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
}

fn main() {
  let system = System::new_all();
  let new_all = system.processes_by_name("PwccaAuto");
  for i in new_all {
    if std::process::id() != i.pid().as_u32() {
      std::process::exit(0);
    }
  }

  println!("Running Pwcca Auto");

  let (sender, receiver) = std::sync::mpsc::channel::<Events>();

  let mut tray_icon = TrayIconBuilder::new()
    .sender(move |e: &Events| sender.send(*e).unwrap())
    .icon_from_buffer(include_bytes!("../res/icon.ico"))
    .tooltip("Pwcca Auto")
    .on_right_click(Events::RightClickTrayIcon)
    .build()
    .unwrap();

  setup_tray_icon_menu(&mut tray_icon);

  let _ = std::thread::Builder::new().name("Media_Thread".to_string()).spawn(media_thread);
  let _ = std::thread::Builder::new()
    .name("Connection_Thread".to_string())
    .spawn(connection_thread);
  let _ = std::thread::Builder::new()
    .name("Tray_Thread".to_string())
    .spawn(|| tray_thread(receiver, tray_icon));

  // Application loop
  loop {
    unsafe {
      let mut msg = MaybeUninit::uninit();
      let bret = GetMessageA(msg.as_mut_ptr(), None, 0, 0);
      if bret.0 > 0 {
        let _ = TranslateMessage(msg.as_ptr());
        DispatchMessageA(msg.as_ptr());
      } else {
        break;
      }
    }
  }
}

#[allow(dead_code)]
fn tray_thread(receiver: std::sync::mpsc::Receiver<Events>, mut tray_icon: trayicon::TrayIcon<Events>) {
  // Initialize the tray thread
  println!("  + Running Tray Thread");

  receiver.iter().for_each(|m| match m {
    Events::RightClickTrayIcon => {
      tray_icon.show_menu().unwrap();
    }
    Events::Discord => unsafe {
      DISCORD = !DISCORD;
      setup_tray_icon_menu(&mut tray_icon);
    },
    Events::Ethernet => unsafe {
      ETHERNET = !ETHERNET;
      setup_tray_icon_menu(&mut tray_icon);
    },
    Events::TurnOffMonitor => turn_off_monitor(),
    Events::RefreshRate => {
      let refresh_rate = get_current_frequency();
      let max_refresh_rate = get_all_frequencies().last().copied().unwrap();
      set_new_frequency(if refresh_rate == 60 { max_refresh_rate } else { 60 });

      setup_tray_icon_menu(&mut tray_icon);
    }
    Events::Exit => std::process::exit(0),
  });
}

#[allow(dead_code)]
fn media_thread() -> Result<(), AudioDeviceError> {
  // Initialize the media thread
  println!("  + Running Media Thread");

  init()?;

  let mut connected = false;
  let discord_executable = String::from("Discord.exe");

  loop {
    if unsafe { DISCORD } {
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
            let headphones = all_outputs.iter().find(|device| device.device_type == "Headphones").unwrap();

            change_default_output(headphones.device_id)?
          }
        } else if connected {
          connected = false;

          // Switch back to speakers if Discord is not recording and headphones are the default
          if current_output.device_type == "Headphones" {
            let headphones = all_outputs.iter().find(|device| device.device_type == "Speakers").unwrap();

            change_default_output(headphones.device_id)?
          }
        }
      }
    }

    std::thread::sleep(Duration::from_secs(1));
  }
}

#[allow(dead_code)]
fn connection_thread() -> Result<(), AudioDeviceError> {
  // Initialize the connection thread
  println!("  + Running Connection Thread");

  loop {
    if unsafe { ETHERNET } {
      let _ = set_wifi_state(!is_ethernet_plugged_in());
    }

    std::thread::sleep(Duration::from_secs(1));
  }
}
