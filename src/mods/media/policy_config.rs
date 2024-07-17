// https://github.com/rkarp/winapi-easy/blob/master/src/media.rs#L172-L250

#![allow(non_upper_case_globals, non_snake_case)]

use std::ffi::c_void;

use windows::core::{imp, IUnknown_Vtbl, Interface, Result, GUID, PCWSTR};
use windows::Win32::Media::Audio::ERole;

pub(crate) mod com;
use com::ComInterfaceExt;
use windows_core::{IUnknown, HRESULT};

#[repr(transparent)]
pub struct IPolicyConfig(IUnknown);

impl IPolicyConfig {
  pub unsafe fn SetDefaultEndpoint<P0, P1>(&self, device_id: P0, e_role: P1) -> Result<()>
  where
    P0: Into<PCWSTR>,
    P1: Into<ERole>,
  {
    (Interface::vtable(self).SetDefaultEndpoint)(Interface::as_raw(self), device_id.into(), e_role.into()).ok()
  }
}

imp::interface_hierarchy!(IPolicyConfig, IUnknown);

impl Clone for IPolicyConfig {
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
}
impl PartialEq for IPolicyConfig {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}
impl Eq for IPolicyConfig {}
impl core::fmt::Debug for IPolicyConfig {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_tuple("IPolicyConfig").field(&self.0).finish()
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
  padding: [*const c_void; 10], // Other fns may be added later
  pub SetDefaultEndpoint: unsafe extern "system" fn(this: *mut c_void, wszDeviceId: PCWSTR, eRole: ERole) -> HRESULT,
  padding2: [*const c_void; 1], // Other fns may be added later
}

const CPolicyConfigClient: GUID = GUID::from_u128(0x870af99c_171d_4f9e_af0d_e63df40c2bc9);

impl ComInterfaceExt for IPolicyConfig {
  const CLASS_GUID: GUID = CPolicyConfigClient;
}
