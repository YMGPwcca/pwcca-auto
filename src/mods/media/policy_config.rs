// https://github.com/rkarp/winapi-easy/blob/master/src/media.rs#L172-L250

#![allow(non_upper_case_globals, non_snake_case)]

use std::ffi::c_void;
use windows::core::{IUnknown, HRESULT};
use windows::core::{IUnknown_Vtbl, Interface, Result, GUID, PCWSTR};
use windows::Win32::Media::Audio::ERole;
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};

#[repr(transparent)]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct IPolicyConfig(pub IUnknown);
impl IPolicyConfig {
  pub fn new() -> std::io::Result<Self> {
    unsafe {
      CoCreateInstance(&GUID::from_u128(0x870af99c_171d_4f9e_af0d_e63df40c2bc9), None, CLSCTX_INPROC_SERVER).map_err(Into::into)
      // .map(Self)
    }
  }

  pub unsafe fn SetDefaultEndpoint(&self, device_id: PCWSTR, e_role: ERole) -> Result<()> {
    (Interface::vtable(self).SetDefaultEndpoint)(Interface::as_raw(self), device_id, e_role).ok()
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
