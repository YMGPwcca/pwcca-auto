use std::fmt::{Display, Formatter, Result};

use windows::{core::PWSTR, Win32::Media::Audio::IMMDevice};

#[derive(Debug)]
pub struct Device {
  pub device_object: IMMDevice,
  pub device_id: PWSTR,
  pub device_type: String,
  pub device_name: String,
}
impl Display for Device {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Name: {}\t Type: {}", self.device_name, self.device_type)
  }
}

#[derive(Clone, Debug)]
pub enum DeviceType {
  Input,
  Output,
}

#[derive(Debug)]
pub enum AudioDeviceError {
  InitializationFailed,
  DeviceNotFound,
  GetDeviceCollectionFailed,
  GetDeviceIdFailed,
  OpenPropertyStoreFailed,
  GetPropertyStoreValueFailed,
  SetDefaultEndpointFailed,

  CreateCOMObjectFailed,
  GetSessionEnumeratorFailed,
  GetSessionFailed,
  CastFailed,

  GetStateFailed,
  GetProcessIdFailed,
}

impl Display for AudioDeviceError {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result {
    match self {
      AudioDeviceError::InitializationFailed => write!(f, "Failed to initialize audio device"),
      AudioDeviceError::DeviceNotFound => write!(f, "Audio device not found"),
      AudioDeviceError::GetDeviceCollectionFailed => write!(f, "Failed to get device collection"),
      AudioDeviceError::GetDeviceIdFailed => write!(f, "Failed to get device ID"),
      AudioDeviceError::OpenPropertyStoreFailed => write!(f, "Failed to open property store"),
      AudioDeviceError::GetPropertyStoreValueFailed => write!(f, "Failed to get property store value"),
      AudioDeviceError::SetDefaultEndpointFailed => write!(f, "Failed to set default endpoint"),

      AudioDeviceError::CreateCOMObjectFailed => write!(f, "Failed to create COM object"),
      AudioDeviceError::GetSessionEnumeratorFailed => write!(f, "Failed to get session enumerator"),
      AudioDeviceError::GetSessionFailed => write!(f, "Failed to get session"),
      AudioDeviceError::CastFailed => write!(f, "Failed to cast interface"),

      AudioDeviceError::GetStateFailed => write!(f, "Failed to get state"),
      AudioDeviceError::GetProcessIdFailed => write!(f, "Failed to get process ID"),
    }
  }
}
