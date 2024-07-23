use windows::core::Error as wError;

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
    Self {
      kind,
      error: error.message(),
    }
  }

  pub fn new_with_message(kind: ErrorEnum, error: String) -> Self {
    Self { kind, error }
  }
}
