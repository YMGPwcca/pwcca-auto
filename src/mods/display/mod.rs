use windows::{
  core::PCWSTR,
  Win32::{
    Foundation::{LPARAM, WPARAM},
    Graphics::Gdi::{
      ChangeDisplaySettingsW, EnumDisplaySettingsW, CDS_GLOBAL, CDS_UPDATEREGISTRY, DEVMODEW,
      DISP_CHANGE_SUCCESSFUL, ENUM_CURRENT_SETTINGS, ENUM_DISPLAY_SETTINGS_MODE,
    },
    UI::WindowsAndMessaging::{GetForegroundWindow, SendMessageW, SC_MONITORPOWER, WM_SYSCOMMAND},
  },
};

#[allow(dead_code)]
fn get_dev_mode() -> DEVMODEW {
  unsafe {
    let mut dev_mode = std::mem::zeroed();
    EnumDisplaySettingsW(PCWSTR::null(), ENUM_CURRENT_SETTINGS, &mut dev_mode).unwrap();

    dev_mode
  }
}

#[allow(dead_code)]
pub fn get_all_frequencies() -> Vec<u32> {
  let mut frequency_vec = Vec::<u32>::new();
  unsafe {
    let mut dev_mode = std::mem::zeroed();
    let mut index = 0;

    loop {
      if EnumDisplaySettingsW(
        PCWSTR::null(),
        ENUM_DISPLAY_SETTINGS_MODE(index),
        &mut dev_mode,
      ) == false
      {
        break;
      }

      if !frequency_vec.contains(&dev_mode.dmDisplayFrequency) {
        frequency_vec.push(dev_mode.dmDisplayFrequency)
      }
      index += 1;
    }

    frequency_vec.sort();
    frequency_vec
  }
}

#[allow(dead_code)]
pub fn get_current_frequency() -> u32 {
  get_dev_mode().dmDisplayFrequency
}

#[allow(dead_code)]
pub fn set_new_frequency(mut frequency: u32) {
  if frequency < 60 {
    frequency = 60;
  }

  let max_frequency = get_all_frequencies().last().copied().unwrap();
  if frequency > max_frequency {
    frequency = max_frequency;
  }

  let dev_mode = DEVMODEW {
    dmDisplayFrequency: frequency,
    ..get_dev_mode()
  };

  unsafe {
    let result = ChangeDisplaySettingsW(Some(&dev_mode), CDS_GLOBAL | CDS_UPDATEREGISTRY);
    if result != DISP_CHANGE_SUCCESSFUL {
      panic!("[PCM] Unable to change display settings!");
    }
  }
}

#[allow(dead_code)]
pub fn turn_off_monitor() {
  unsafe {
    SendMessageW(
      GetForegroundWindow(),
      WM_SYSCOMMAND,
      WPARAM(SC_MONITORPOWER as usize),
      LPARAM(2isize),
    )
  };
}
