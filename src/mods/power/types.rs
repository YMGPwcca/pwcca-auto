use windows::core::GUID;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SystemPowerStatus {
  pub is_plugged_in: bool,
  pub is_battery_saver_enabled: bool,
  pub remaining_percentage: u32,
  pub remaining_time: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PowerScheme {
  pub name: String,
  pub guid: GUID,
}
