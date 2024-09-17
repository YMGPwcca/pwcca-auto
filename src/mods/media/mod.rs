#![allow(dead_code)]

mod policy_config;
pub mod types;

use std::{path::Path, str::FromStr};

use types::{
  device::{Device, DeviceType},
  error::{AudioDeviceError, ErrorEnum},
};
use windows::{
  core::{Interface, PCWSTR, PWSTR},
  Win32::{
    Devices::FunctionDiscovery::{PKEY_DeviceInterface_FriendlyName, PKEY_Device_DeviceDesc},
    Foundation::{CloseHandle, MAX_PATH, S_OK},
    Media::Audio::{
      eCapture, eCommunications, eConsole, eRender, AudioSessionStateActive, IAudioSessionControl2,
      IAudioSessionManager2, IMMDevice, IMMDeviceEnumerator, MMDeviceEnumerator,
      DEVICE_STATE_ACTIVE,
    },
    System::{
      Com::{CoCreateInstance, CoInitialize, CLSCTX_ALL, STGM_READ},
      ProcessStatus::GetProcessImageFileNameW,
      Threading::{OpenProcess, PROCESS_ALL_ACCESS},
    },
  },
};

static mut IS_INITIALIZED: bool = false;

pub fn init() -> Result<(), AudioDeviceError> {
  unsafe {
    if IS_INITIALIZED {
      return Ok(());
    }

    let res = CoInitialize(None);
    if res.is_err() {
      return Err(AudioDeviceError::new(
        ErrorEnum::InitializationFailed,
        res.into(),
      ));
    }
    IS_INITIALIZED = true;
    Ok(())
  }
}

fn init_check() -> Result<(), AudioDeviceError> {
  if !unsafe { IS_INITIALIZED } {
    return Err(AudioDeviceError::new_with_message(
      ErrorEnum::NotInitialized,
      "Audio device not initialized.".to_string(),
    ));
  }
  Ok(())
}

fn get_device_info(device: &IMMDevice) -> Result<Device, AudioDeviceError> {
  unsafe {
    let property_store = device
      .OpenPropertyStore(STGM_READ)
      .map_err(|e| AudioDeviceError::new(ErrorEnum::OpenPropertyStoreFailed, e))?;
    let device_id = device
      .GetId()
      .map_err(|e| AudioDeviceError::new(ErrorEnum::GetDeviceIdFailed, e))?;
    let device_type = property_store
      .GetValue(&PKEY_Device_DeviceDesc)
      .map_err(|e| AudioDeviceError::new(ErrorEnum::GetPropertyStoreValueFailed, e))?
      .to_string();
    let device_name = property_store
      .GetValue(&PKEY_DeviceInterface_FriendlyName)
      .map_err(|e| AudioDeviceError::new(ErrorEnum::GetPropertyStoreValueFailed, e))?
      .to_string();

    Ok(Device {
      device_object: device.clone(),
      device_id,
      device_type,
      device_name,
    })
  }
}

pub fn get_default_device(device_type: &DeviceType) -> Result<Device, AudioDeviceError> {
  init_check()?;

  unsafe {
    let enumerator: IMMDeviceEnumerator =
      CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
        .map_err(|e| AudioDeviceError::new(ErrorEnum::InitializationFailed, e))?;
    let device = match device_type {
      DeviceType::Input => enumerator
        .GetDefaultAudioEndpoint(eCapture, eCommunications)
        .map_err(|e| AudioDeviceError::new(ErrorEnum::DeviceNotFound, e))?,
      DeviceType::Output => enumerator
        .GetDefaultAudioEndpoint(eRender, eConsole)
        .map_err(|e| AudioDeviceError::new(ErrorEnum::DeviceNotFound, e))?,
    };

    drop(enumerator);

    get_device_info(&device)
  }
}

pub fn enumerate_audio_devices(device_type: &DeviceType) -> Result<Vec<Device>, AudioDeviceError> {
  init_check()?;

  let mut all_devices = Vec::<Device>::new();

  unsafe {
    let enumerator: IMMDeviceEnumerator =
      CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
        .map_err(|e| AudioDeviceError::new(ErrorEnum::InitializationFailed, e))?;
    let devices = match device_type {
      DeviceType::Input => enumerator
        .EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE)
        .map_err(|e| AudioDeviceError::new(ErrorEnum::GetDeviceCollectionFailed, e))?,
      DeviceType::Output => enumerator
        .EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)
        .map_err(|e| AudioDeviceError::new(ErrorEnum::GetDeviceCollectionFailed, e))?,
    };
    for i in 0..devices.GetCount().unwrap() {
      let device = devices
        .Item(i)
        .map_err(|e| AudioDeviceError::new(ErrorEnum::DeviceNotFound, e))?;
      all_devices.push(get_device_info(&device)?);

      drop(device);
    }

    drop(enumerator);
    drop(devices);

    Ok(all_devices)
  }
}

pub fn change_default_output(device_id: PWSTR) -> Result<(), AudioDeviceError> {
  init_check()?;

  unsafe {
    let policy = policy_config::IPolicyConfig::new()
      .map_err(|e| AudioDeviceError::new(ErrorEnum::InitializationFailed, e.into()))?;
    policy
      .SetDefaultEndpoint(PCWSTR(device_id.as_ptr()), eConsole)
      .map_err(|e| AudioDeviceError::new(ErrorEnum::SetDefaultEndpointFailed, e))?;

    drop(policy);

    Ok(())
  }
}

fn get_process_name(process_id: u32) -> Result<String, AudioDeviceError> {
  let h_process = unsafe { OpenProcess(PROCESS_ALL_ACCESS, false, process_id) }.unwrap_or_default();

  if !h_process.is_invalid() {
    let mut process_path_buffer = [0; MAX_PATH as usize];
    let byte_written = unsafe { GetProcessImageFileNameW(h_process, &mut process_path_buffer) };

    unsafe { CloseHandle(h_process).unwrap() };
    if byte_written == 0 {
      return Ok(String::new());
    };

    let process_path =
      String::from_utf16(&process_path_buffer[..byte_written as usize]).unwrap_or_default();
    let process_name = String::from_str(
      Path::new(&process_path)
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default(),
    )
    .unwrap_or_default()
    .to_lowercase();

    return Ok(process_name);
  }

  Ok(String::new())
}

pub fn get_active_audio_applications(
  device_type: &DeviceType,
) -> Result<Vec<String>, AudioDeviceError> {
  init_check()?;

  let mut result = Vec::<String>::new();
  let device = get_default_device(device_type)?;

  unsafe {
    let session_manager: IAudioSessionManager2 = device
      .device_object
      .Activate(CLSCTX_ALL, None)
      .map_err(|e| AudioDeviceError::new(ErrorEnum::CreateCOMObjectFailed, e))?;
    let session_list = session_manager
      .GetSessionEnumerator()
      .map_err(|e| AudioDeviceError::new(ErrorEnum::GetSessionEnumeratorFailed, e))?;

    for i in 0..session_list.GetCount().unwrap() {
      let session_control = session_list
        .GetSession(i)
        .map_err(|e| AudioDeviceError::new(ErrorEnum::GetSessionFailed, e))?;
      let session_control2: IAudioSessionControl2 = session_control
        .cast()
        .map_err(|e| AudioDeviceError::new(ErrorEnum::CastFailed, e))?;

      if session_control2.IsSystemSoundsSession() == S_OK {
        continue;
      }

      let state = session_control2
        .GetState()
        .map_err(|e| AudioDeviceError::new(ErrorEnum::GetStateFailed, e))?;
      if state == AudioSessionStateActive {
        let instance_id = session_control2
          .GetProcessId()
          .map_err(|e| AudioDeviceError::new(ErrorEnum::GetProcessIdFailed, e))?;
        result.push(get_process_name(instance_id)?);
      }

      drop(session_control);
      drop(session_control2);
    }

    drop(device);
    drop(session_manager);
    drop(session_list);

    Ok(result)
  }
}
