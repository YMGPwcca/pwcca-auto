use std::fmt::Display;

use windows::{
  core::{Error as wError, PWSTR},
  Win32::Media::Audio::IMMDevice,
};

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

#[derive(Clone, Copy, Debug)]
pub enum ErrorEnum {
  NotInitialized,

  InitializationFailed,
  OpenPropertyStoreFailed,
  GetPropertyStoreValueFailed,
  GetDeviceIdFailed,
  DeviceNotFound,
  GetDeviceCollectionFailed,
  SetDefaultEndpointFailed,
  CreateCOMObjectFailed,
  GetSessionEnumeratorFailed,
  GetSessionFailed,

  CastFailed,
  GetStateFailed,
  GetProcessIdFailed,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct AudioDeviceError {
  kind: ErrorEnum,
  error: String,
}

#[allow(dead_code)]
impl AudioDeviceError {
  pub fn new(kind: ErrorEnum, error: wError) -> Self {
    Self { kind, error: error.message() }
  }

  pub fn new_with_message(kind: ErrorEnum, error: String) -> Self {
    Self { kind, error }
  }
}
