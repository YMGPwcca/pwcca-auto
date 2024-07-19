mod policy_config;

use std::{fmt::Display, path::Path, str::FromStr};
use windows::{
  core::{Interface, PCWSTR, PWSTR},
  Win32::{
    Devices::FunctionDiscovery::{PKEY_DeviceInterface_FriendlyName, PKEY_Device_DeviceDesc},
    Foundation::{CloseHandle, MAX_PATH, S_OK},
    Media::Audio::{
      eCapture, eCommunications, eConsole, eRender, AudioSessionStateActive, IAudioSessionControl2,
      IAudioSessionManager2, IMMDevice, IMMDeviceEnumerator, MMDeviceEnumerator, DEVICE_STATE_ACTIVE,
    },
    System::{
      Com::{CoCreateInstance, CoInitialize, CLSCTX_ALL, STGM_READ},
      ProcessStatus::GetProcessImageFileNameA,
      Threading::{OpenProcess, PROCESS_ALL_ACCESS},
    },
  },
};

static mut IS_INITIALIZED: bool = false;

#[derive(Clone)]
pub struct Device {
  device_object: IMMDevice,
  pub device_id: PWSTR,
  pub device_type: String,
  pub device_name: String,
}
impl PartialEq for Device {
  fn eq(&self, other: &Self) -> bool {
    unsafe {
      self.device_id.to_string().unwrap() == other.device_id.to_string().unwrap()
        && self.device_name == other.device_name
    }
  }
}
impl Eq for Device {}
impl Display for Device {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Name: {}\t Type: {}", self.device_name, self.device_type)
  }
}

#[allow(dead_code)]
pub fn init() {
  unsafe {
    let res = CoInitialize(None);
    if res.is_ok() {
      IS_INITIALIZED = true;
    }
  }
}

#[allow(dead_code)]
fn init_check() {
  if !unsafe { IS_INITIALIZED } {
    panic!("[PCM] CoInitialize has not been called. Consider calling `init` function.")
  }
}

#[allow(dead_code)]
pub fn get_default_input() -> Device {
  unsafe {
    init_check();

    let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).unwrap();
    let device = enumerator.GetDefaultAudioEndpoint(eCapture, eCommunications).unwrap();

    let property_store = device.OpenPropertyStore(STGM_READ).unwrap();
    let device_id = device.clone().GetId().unwrap();
    let device_type = property_store.GetValue(&PKEY_Device_DeviceDesc).unwrap().to_string();
    let device_name = property_store
      .GetValue(&PKEY_DeviceInterface_FriendlyName)
      .unwrap()
      .to_string();

    Device {
      device_object: device,
      device_type,
      device_id,
      device_name,
    }
  }
}

#[allow(dead_code)]
pub fn get_default_output() -> Device {
  unsafe {
    init_check();

    let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).unwrap();
    let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole).unwrap();

    let property_store = device.OpenPropertyStore(STGM_READ).unwrap();
    let device_id = device.clone().GetId().unwrap();
    let device_type = property_store.GetValue(&PKEY_Device_DeviceDesc).unwrap().to_string();
    let device_name = property_store
      .GetValue(&PKEY_DeviceInterface_FriendlyName)
      .unwrap()
      .to_string();

    Device {
      device_object: device,
      device_type,
      device_id,
      device_name,
    }
  }
}

#[allow(dead_code)]
pub fn get_all_outputs() -> Vec<Device> {
  init_check();
  let mut all_outputs = Vec::<Device>::new();

  unsafe {
    let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).unwrap();
    let devices = enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE).unwrap();
    let count = devices.GetCount().unwrap();
    for i in 0..count {
      let device = devices.Item(i).unwrap();
      let property_store = device.OpenPropertyStore(STGM_READ).unwrap();

      let device_id = device.GetId().unwrap();
      let device_type = property_store.GetValue(&PKEY_Device_DeviceDesc).unwrap().to_string();
      let device_name = property_store
        .GetValue(&PKEY_DeviceInterface_FriendlyName)
        .unwrap()
        .to_string();

      all_outputs.push(Device {
        device_object: device,
        device_type,
        device_id,
        device_name,
      })
    }

    all_outputs
  }
}

#[allow(dead_code)]
pub fn get_all_inputs() -> Vec<Device> {
  init_check();
  let mut all_inputs = Vec::<Device>::new();

  unsafe {
    let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).unwrap();
    let devices = enumerator.EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE).unwrap();
    let count = devices.GetCount().unwrap();
    for i in 0..count {
      let device = devices.Item(i).unwrap();
      let property_store = device.OpenPropertyStore(STGM_READ).unwrap();

      let device_id = device.GetId().unwrap();
      let device_type = property_store.GetValue(&PKEY_Device_DeviceDesc).unwrap().to_string();
      let device_name = property_store
        .GetValue(&PKEY_DeviceInterface_FriendlyName)
        .unwrap()
        .to_string();

      all_inputs.push(Device {
        device_object: device,
        device_type,
        device_id,
        device_name,
      })
    }

    all_inputs
  }
}

#[allow(dead_code)]
pub fn change_default_output(device_id: PWSTR) {
  unsafe {
    init_check();

    let policy = policy_config::IPolicyConfig::new_instance().unwrap();
    policy.SetDefaultEndpoint(PCWSTR(device_id.as_ptr()), eConsole).unwrap()
  };
}

#[allow(dead_code)]
fn get_process_name(process_id: u32) -> String {
  let h_process = unsafe { OpenProcess(PROCESS_ALL_ACCESS, false, process_id) }.unwrap();

  if !h_process.is_invalid() {
    let mut process_path_buffer = [0; MAX_PATH as usize];
    let byte_written = unsafe { GetProcessImageFileNameA(h_process, &mut process_path_buffer) as usize };

    if byte_written == 0 {
      return String::new();
    };

    let process_path = String::from_utf8(process_path_buffer[..byte_written].to_vec()).unwrap();
    let process_name = String::from_str(Path::new(&process_path).file_name().unwrap().to_str().unwrap()).unwrap();

    return process_name;
  }

  unsafe { CloseHandle(h_process).unwrap() };

  String::new()
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
  Input,
  Output,
}

#[allow(dead_code)]
pub fn list_all_programs(device_type: &DeviceType) -> Vec<String> {
  let mut result = Vec::<String>::new();

  init_check();
  let device = if *device_type == DeviceType::Input {
    get_default_input()
  } else {
    get_default_output()
  };

  unsafe {
    let session_manager: IAudioSessionManager2 = device.device_object.Activate(CLSCTX_ALL, None).unwrap();
    let session_list = session_manager.GetSessionEnumerator().unwrap();

    for i in 0..session_list.GetCount().unwrap() {
      let session_control = session_list.GetSession(i).unwrap();
      let session_control2: IAudioSessionControl2 = session_control.cast().unwrap();

      if session_control2.IsSystemSoundsSession() == S_OK {
        continue;
      }

      let state = session_control2.GetState().unwrap();
      if state == AudioSessionStateActive {
        let instance_id = session_control2.GetProcessId().unwrap();
        result.push(get_process_name(instance_id));
      }
    }

    result
  }
}
