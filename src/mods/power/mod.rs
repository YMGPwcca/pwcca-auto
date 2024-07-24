use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};

#[allow(dead_code)]
pub struct SystemPowerStatus {
  is_plugged_in: bool,
  is_battery_saver_enabled: bool,
  remaining_percentage: u8,
  remaining_time: u32,
}

#[allow(dead_code)]
pub fn get_power_status() -> SystemPowerStatus {
  unsafe {
    let mut system_power_status = SYSTEM_POWER_STATUS::default();
    GetSystemPowerStatus(&mut system_power_status).unwrap();

    SystemPowerStatus {
      is_plugged_in: system_power_status.ACLineStatus == 1,
      is_battery_saver_enabled: system_power_status.SystemStatusFlag == 1,
      remaining_percentage: system_power_status.BatteryLifePercent,
      remaining_time: system_power_status.BatteryLifeTime,
    }
  }
}
