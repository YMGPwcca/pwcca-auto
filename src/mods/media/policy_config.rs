// https://github.com/rkarp/winapi-easy/blob/master/src/media.rs#L172-L250

#![allow(non_upper_case_globals, non_snake_case)]

use std::ffi::c_void;
use windows::core::{IUnknown, HRESULT};
use windows::core::{IUnknown_Vtbl, Interface, Result, GUID, PCWSTR};
use windows::Win32::Media::Audio::ERole;
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED};

static mut COM_INITIALIZED: bool = false;

#[repr(transparent)]
#[derive(Clone)]
pub struct IPolicyConfig(IUnknown);
impl IPolicyConfig {
  const CLASS_GUID: GUID = GUID::from_u128(0x870af99c_171d_4f9e_af0d_e63df40c2bc9);

  pub fn new_instance() -> std::io::Result<Self> {
    if !unsafe { COM_INITIALIZED } {
      let init_result = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok() };
      if let Ok(()) = init_result {
        unsafe { COM_INITIALIZED = true };
      }
    }

    let result = unsafe { CoCreateInstance(&Self::CLASS_GUID, None, CLSCTX_INPROC_SERVER) };
    result.map_err(Into::into)
  }

  pub unsafe fn SetDefaultEndpoint(&self, device_id: PCWSTR, e_role: ERole) -> Result<()> {
    (Interface::vtable(self).SetDefaultEndpoint)(Interface::as_raw(self), device_id.into(), e_role.into()).ok()
  }
}

unsafe impl Interface for IPolicyConfig {
  type Vtable = IPolicyConfig_Vtbl;
  const IID: GUID = GUID::from_u128(0xf8679f50_850a_41cf_9c72_430f290290c8);
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct IPolicyConfig_Vtbl {
  pub base__: IUnknown_Vtbl,
  padding: [*const c_void; 10],
  pub SetDefaultEndpoint: unsafe extern "system" fn(this: *mut c_void, wszDeviceId: PCWSTR, eRole: ERole) -> HRESULT,
}
