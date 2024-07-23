use windows::Win32::Foundation::{
  ERROR_INVALID_HANDLE, ERROR_INVALID_PARAMETER, ERROR_NOT_ENOUGH_MEMORY,
  ERROR_REMOTE_SESSION_LIMIT_EXCEEDED, WIN32_ERROR,
};

#[derive(Clone, Copy, Debug)]
pub enum ErrorEnum {
  // General
  InvalidParameters,
  NotEnoughMemory,

  // WlanOpenHandle
  RemoteSessionLimitExceeded,

  // WlanEnumInterfaces
  InvalidHandle,

  Unknown,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct WlanHandlerError {
  kind: ErrorEnum,
  error: String,
}

#[allow(dead_code)]
impl WlanHandlerError {
  pub fn new(kind: WIN32_ERROR) -> Self {
    match kind {
      ERROR_INVALID_PARAMETER => WlanHandlerError {
        kind: ErrorEnum::InvalidParameters,
        error: format!("{:?}", "Invalid parameters."),
      },
      ERROR_NOT_ENOUGH_MEMORY => WlanHandlerError {
        kind: ErrorEnum::NotEnoughMemory,
        error: format!("{:?}", "Failed to allocate memory."),
      },
      ERROR_REMOTE_SESSION_LIMIT_EXCEEDED => WlanHandlerError {
        kind: ErrorEnum::RemoteSessionLimitExceeded,
        error: format!("{:?}", "Too many handles have been issued by the server."),
      },
      ERROR_INVALID_HANDLE => WlanHandlerError {
        kind: ErrorEnum::InvalidHandle,
        error: format!("{:?}", "Invalid handle."),
      },
      _ => WlanHandlerError {
        kind: ErrorEnum::Unknown,
        error: format!("Unknown ({:?})", kind),
      },
    }
  }
}
