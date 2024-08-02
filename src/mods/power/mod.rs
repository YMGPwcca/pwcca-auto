mod types;

use std::{ffi::OsString, os::windows::ffi::OsStringExt};
use windows::{
  core::GUID,
  Win32::{
    Foundation::{ERROR_NO_MORE_ITEMS, ERROR_SUCCESS, WIN32_ERROR},
    System::Power::{
      GetSystemPowerStatus, PowerEnumerate, PowerGetActiveScheme, PowerReadFriendlyName,
      PowerSetActiveScheme, ACCESS_SCHEME, SYSTEM_POWER_STATUS,
    },
  },
};

use types::{PowerScheme, SystemPowerStatus};

#[allow(dead_code)]
pub fn get_power_status() -> SystemPowerStatus {
  unsafe {
    let mut system_power_status = SYSTEM_POWER_STATUS::default();
    GetSystemPowerStatus(&mut system_power_status).unwrap();

    SystemPowerStatus {
      is_plugged_in: system_power_status.ACLineStatus == 1,
      is_battery_saver_enabled: system_power_status.SystemStatusFlag == 1,
      remaining_percentage: system_power_status.BatteryLifePercent as u32,
      remaining_time: system_power_status.BatteryLifeTime,
    }
  }
}

#[allow(dead_code)]
pub fn get_all_power_schemes() -> Result<Vec<PowerScheme>, WIN32_ERROR> {
  let mut power_schemes = Vec::new();
  let mut index = 0;
  let mut buffersize = std::mem::size_of::<GUID>() as u32;

  loop {
    unsafe {
      let mut buffer: GUID = std::mem::zeroed();

      let result = PowerEnumerate(
        None,
        None,
        None,
        ACCESS_SCHEME,
        index,
        Some(&mut buffer as *mut _ as *mut u8),
        &mut buffersize,
      );

      if result != ERROR_SUCCESS && result != ERROR_NO_MORE_ITEMS {
        return Err(result);
      }

      if result == ERROR_SUCCESS {
        power_schemes.push(PowerScheme {
          name: get_power_scheme_friendly_name(&buffer).unwrap(),
          guid: buffer,
        });
        index += 1;
      } else {
        break;
      }
    };
  }

  Ok(power_schemes)
}

#[allow(dead_code)]
pub fn get_active_power_scheme() -> Result<PowerScheme, WIN32_ERROR> {
  let mut buffer = std::ptr::null_mut();

  unsafe {
    let result = PowerGetActiveScheme(None, &mut buffer);
    if result == ERROR_SUCCESS {
      Ok(PowerScheme {
        name: get_power_scheme_friendly_name(&*buffer)?,
        guid: *buffer,
      })
    } else {
      Err(result)
    }
  }
}

#[allow(dead_code)]
pub fn set_active_power_scheme(guid: &GUID) -> Result<(), WIN32_ERROR> {
  let result = unsafe { PowerSetActiveScheme(None, Some(guid)) };
  if result == ERROR_SUCCESS {
    Ok(())
  } else {
    Err(result)
  }
}

#[allow(dead_code)]
fn get_power_scheme_friendly_name(scheme_guid: &GUID) -> Result<String, WIN32_ERROR> {
  let mut buffer_size: u32 = 1024; // Maximum buffer size during testing is 258, hopefully it won't break anytime soon
  let mut buffer: Vec<u8> = Vec::with_capacity(buffer_size as usize);

  unsafe {
    let result = PowerReadFriendlyName(
      None,
      Some(scheme_guid),
      None,
      None,
      Some(buffer.as_mut_ptr()),
      &mut buffer_size,
    );
    if result != ERROR_SUCCESS {
      return Err(result);
    }

    let os_str = OsString::from_wide(std::slice::from_raw_parts(
      buffer.as_ptr() as *const u16,
      buffer_size as usize / 2,
    ));
    match os_str.to_string_lossy().to_string() {
      string if !string.is_empty() => Ok(string.trim_end_matches('\0').to_string()),
      _ => Err(result),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_get_power_status() {
    let status = get_power_status();

    println!("{:#?}", status);
  }

  #[test]
  fn test_get_all_power_schemes() {
    let schemes = get_all_power_schemes().unwrap();

    println!("{:#?}", schemes);
  }

  #[test]
  fn test_get_active_power_scheme() {
    let scheme = get_active_power_scheme().unwrap();

    println!("{:#?}", scheme);
  }

  #[test]
  fn test_set_active_power_scheme() {
    let all_schemes = get_all_power_schemes().unwrap();
    let active_scheme = get_active_power_scheme().unwrap();

    let without_active = all_schemes
      .iter()
      .filter(|&scheme| scheme.guid != active_scheme.guid)
      .collect::<Vec<&PowerScheme>>();

    let selected = without_active[0];
    let result = set_active_power_scheme(&selected.guid);
    println!("Switched to: {}", &selected.name);
    assert!(result.is_ok())
  }
}
