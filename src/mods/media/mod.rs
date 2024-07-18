mod policy_config;

use policy_config::com::ComInterfaceExt;
use std::{
  ffi::OsString,
  iter::once,
  ops::Deref,
  os::windows::ffi::{OsStrExt, OsStringExt},
};
use windows::{
  core::{Interface, PWSTR},
  Win32::{
    Devices::FunctionDiscovery::PKEY_Device_FriendlyName,
    Foundation::S_OK,
    Media::Audio::{
      eCapture, eCommunications, eConsole, eRender, AudioSessionStateActive, IAudioSessionControl2,
      IAudioSessionManager2, IMMDevice, IMMDeviceEnumerator, MMDeviceEnumerator, DEVICE_STATE_ACTIVE,
    },
    System::Com::{CoCreateInstance, CoInitialize, CLSCTX_ALL, STGM_READ},
  },
};
use windows_core::PCWSTR;

#[derive(Clone)]
pub struct Device {
  pub device_object: IMMDevice,
  pub device_id: PWSTR,
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

static mut IS_INITIALIZED: bool = false;

#[allow(dead_code)]
pub fn init() {
  unsafe {
    let res = CoInitialize(Some(std::ptr::null()));
    if res.is_ok() {
      IS_INITIALIZED = true;
    }
  }
}

#[allow(dead_code)]
fn init_check() {
  if !unsafe { IS_INITIALIZED } {
    panic!("CoInitialize has not been called. Consider calling `init` function.")
  }
}

// Audio Input
#[allow(dead_code)]
pub fn get_current_input() -> Device {
  unsafe {
    init_check();

    let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).unwrap();
    let device = enumerator.GetDefaultAudioEndpoint(eCapture, eCommunications).unwrap();

    let property_store = device.OpenPropertyStore(STGM_READ).unwrap();
    let id = device.clone().GetId().unwrap();
    let name = property_store.GetValue(&PKEY_Device_FriendlyName).unwrap().to_string();

    return Device {
      device_object: device,
      device_id: id,
      device_name: name,
    };
  }
}

#[allow(dead_code)]
pub fn change_default_output(device_id: PWSTR) {
  unsafe {
    init_check();

    let id = OsString::from_wide(device_id.as_wide());
    let raw_id = id.as_os_str().encode_wide().chain(once(0)).collect::<Vec<u16>>();

    let policy = policy_config::IPolicyConfig::new_instance().unwrap();
    policy.SetDefaultEndpoint(PCWSTR(raw_id.as_ptr()), eConsole).unwrap()
  };
}

// Audio Output
#[allow(dead_code)]
pub fn get_current_output() -> Device {
  unsafe {
    init_check();

    let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).unwrap();
    let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole).unwrap();

    let property_store = device.OpenPropertyStore(STGM_READ).unwrap();
    let id = device.clone().GetId().unwrap();
    let name = property_store.GetValue(&PKEY_Device_FriendlyName).unwrap().to_string();

    return Device {
      device_object: device,
      device_id: id,
      device_name: name,
    };
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

      let id = device.GetId().unwrap();
      let name = property_store.GetValue(&PKEY_Device_FriendlyName).unwrap().to_string();
      all_outputs.push(Device {
        device_object: device,
        device_id: id,
        device_name: name,
      })
    }

    return all_outputs;
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

      let id = device.GetId().unwrap();
      let name = property_store.GetValue(&PKEY_Device_FriendlyName).unwrap().to_string();
      all_inputs.push(Device {
        device_object: device,
        device_id: id,
        device_name: name,
      })
    }

    return all_inputs;
  }
}

#[allow(dead_code)]
pub fn is_discord_recording() -> bool {
  init_check();
  let microphone = get_current_input();

  unsafe {
    let session_manager: IAudioSessionManager2 = microphone
      .device_object
      .Activate(CLSCTX_ALL, Some(std::ptr::null()))
      .unwrap();
    let session_list = session_manager.GetSessionEnumerator().unwrap();
    let session_count = session_list.GetCount().unwrap();

    for i in 0..session_count {
      let session_control = session_list.GetSession(i).unwrap();
      let session_control2: IAudioSessionControl2 = session_control.deref().cast().unwrap();

      if session_control2.IsSystemSoundsSession() == S_OK {
        continue;
      }

      let state = session_control2.GetState().unwrap();
      if state == AudioSessionStateActive {
        let instance_id = session_control2
          .GetSessionInstanceIdentifier()
          .unwrap()
          .to_string()
          .unwrap();
        if instance_id.to_lowercase().contains("discord") {
          return true;
        } else {
          return false;
        }
      }
    }

    return false;
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn get_current_input() {
    println!("{}", super::get_current_input().device_name)
  }

  #[test]
  fn get_current_output() {
    println!("{}", super::get_current_output().device_name)
  }

  #[test]
  fn get_all_inputs() {
    let outputs = super::get_all_inputs();
    for i in 0..outputs.len() {
      let device = outputs.get(i).unwrap();
      println!(
        "{} {}",
        unsafe { device.device_id.to_string().unwrap() },
        device.device_name
      )
    }
  }

  #[test]
  fn get_all_outputs() {
    let outputs = super::get_all_outputs();
    for i in 0..outputs.len() {
      let device = outputs.get(i).unwrap();
      println!(
        "{} {}",
        unsafe { device.device_id.to_string().unwrap() },
        device.device_name
      )
    }
  }

  #[test]
  fn switch_output() {
    let mut outputs = super::get_all_outputs();
    let current = super::get_current_output();

    if outputs.contains(&current) {
      let position = outputs
        .clone()
        .into_iter()
        .position(|device| device == current)
        .unwrap();
      outputs.remove(position);
    }

    super::change_default_output(outputs.get(0).unwrap().device_id);
  }
}
