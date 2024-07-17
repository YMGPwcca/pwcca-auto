mod policy_config {
  #![allow(non_upper_case_globals, non_snake_case)]

  use std::ffi::c_void;
  use windows::core::{Interface, GUID, PCWSTR};
  use windows::Win32::Media::Audio::ERole;

  use crate::com::ComInterfaceExt;

  #[repr(transparent)]
  pub struct IPolicyConfig(windows::core::IUnknown);

  impl IPolicyConfig {
    pub unsafe fn SetDefaultEndpoint<P0, P1>(&self, deviceId: P0, eRole: P1) -> windows::core::Result<()>
    where
      P0: Into<PCWSTR>,
      P1: Into<ERole>,
    {
      (Interface::vtable(self).SetDefaultEndpoint)(Interface::as_raw(self), deviceId.into(), eRole.into()).ok()
    }
  }

  windows::core::imp::interface_hierarchy!(IPolicyConfig, windows::core::IUnknown);

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
    pub base__: windows::core::IUnknown_Vtbl,
    padding: [*const c_void; 10], // Other fns may be added later
    pub SetDefaultEndpoint:
      unsafe extern "system" fn(this: *mut c_void, wszDeviceId: PCWSTR, eRole: ERole) -> windows::core::HRESULT,
    padding2: [*const c_void; 1], // Other fns may be added later
  }

  const CPolicyConfigClient: GUID = GUID::from_u128(0x870af99c_171d_4f9e_af0d_e63df40c2bc9);

  impl ComInterfaceExt for IPolicyConfig {
    const CLASS_GUID: GUID = CPolicyConfigClient;
  }
}
