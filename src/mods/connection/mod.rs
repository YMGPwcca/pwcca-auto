use windows::{
  Devices::Radios::{self, RadioKind},
  Win32::{
    Foundation::{ERROR_SUCCESS, WIN32_ERROR},
    NetworkManagement::{
      IpHelper::{
        GetAdaptersAddresses, GAA_FLAG_SKIP_ANYCAST, GAA_FLAG_SKIP_DNS_SERVER,
        GAA_FLAG_SKIP_MULTICAST, GAA_FLAG_SKIP_UNICAST, IP_ADAPTER_ADDRESSES_LH,
      },
      Ndis::IfOperStatusUp,
    },
  },
};

#[allow(dead_code)]
pub fn is_ethernet_plugged_in() -> bool {
  let mut is_plugged_in = false;

  unsafe {
    // https://docs.microsoft.com/en-us/windows/desktop/api/iphlpapi/nf-iphlpapi-getadaptersaddresses
    let mut buf_len: core::ffi::c_ulong = 16384;
    let mut adapters_addresses_buffer = Vec::with_capacity(buf_len as usize);

    let result = WIN32_ERROR(GetAdaptersAddresses(
      0, // AF_UNSPEC
      GAA_FLAG_SKIP_UNICAST
        | GAA_FLAG_SKIP_ANYCAST
        | GAA_FLAG_SKIP_MULTICAST
        | GAA_FLAG_SKIP_DNS_SERVER,
      None,
      Some(adapters_addresses_buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH),
      &mut buf_len as *mut core::ffi::c_ulong,
    ));

    if result == ERROR_SUCCESS {
      let mut adapter_addresses_ptr =
        adapters_addresses_buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH;

      while !adapter_addresses_ptr.is_null() {
        let adapter = adapter_addresses_ptr.read_unaligned();

        if adapter.IfType == 6
          && adapter.FriendlyName.to_string().unwrap() == "Ethernet"
          && adapter.OperStatus == IfOperStatusUp
        {
          is_plugged_in = true;
        }
        adapter_addresses_ptr = adapter.Next;
      }

      is_plugged_in
    } else {
      false
    }
  }
}

#[allow(dead_code)]
pub fn set_wifi_state(on: bool) -> std::io::Result<()> {
  let radios = Radios::Radio::GetRadiosAsync()?.get()?;
  for radio in 0..radios.Size()? {
    let radio = radios.GetAt(radio)?;

    if radio.Kind()? == RadioKind::WiFi {
      match on {
        true => radio.SetStateAsync(Radios::RadioState::On)?,
        false => radio.SetStateAsync(Radios::RadioState::Off)?,
      };
    }
  }
  Ok(())
}
