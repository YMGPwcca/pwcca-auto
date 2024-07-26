pub mod types;

use types::{error::WlanHandlerError, wlan::WLAN};
use windows::{
  Devices::Radios::{self, RadioKind},
  Win32::{
    Foundation::{ERROR_SUCCESS, HANDLE, WIN32_ERROR},
    NetworkManagement::{
      IpHelper::{
        GetAdaptersAddresses, GAA_FLAG_SKIP_ANYCAST, GAA_FLAG_SKIP_DNS_SERVER, GAA_FLAG_SKIP_MULTICAST, GAA_FLAG_SKIP_UNICAST,
        IP_ADAPTER_ADDRESSES_LH,
      },
      Ndis::IfOperStatusUp,
      WiFi::{
        WlanCloseHandle, WlanEnumInterfaces, WlanFreeMemory, WlanGetAvailableNetworkList, WlanGetNetworkBssList, WlanOpenHandle,
        WLAN_AVAILABLE_NETWORK, WLAN_BSS_ENTRY, WLAN_INTERFACE_INFO,
      },
    },
  },
};

fn open_handle() -> Result<HANDLE, WlanHandlerError> {
  let mut w_handle = HANDLE::default();
  let mut current_version = 0;

  let open_handle_result = WIN32_ERROR(unsafe { WlanOpenHandle(2, None, &mut current_version, &mut w_handle) });
  if open_handle_result != ERROR_SUCCESS {
    return Err(WlanHandlerError::new(open_handle_result));
  }
  Ok(w_handle)
}

fn enum_interfaces(w_handle: &HANDLE) -> Result<Vec<WLAN_INTERFACE_INFO>, WlanHandlerError> {
  unsafe {
    let mut interface_info_list = std::ptr::null_mut();
    let enum_interfaces_result = WIN32_ERROR(WlanEnumInterfaces(*w_handle, None, &mut interface_info_list));
    if enum_interfaces_result != ERROR_SUCCESS {
      return Err(WlanHandlerError::new(enum_interfaces_result));
    }

    // https://stackoverflow.com/a/78779478/9879620
    let interface_info_ptr = std::ptr::addr_of!((*interface_info_list).InterfaceInfo);
    let interface_info_len = (*interface_info_list).dwNumberOfItems as usize;
    let interface_info = std::slice::from_raw_parts(interface_info_ptr.cast::<WLAN_INTERFACE_INFO>(), interface_info_len).to_vec();

    WlanFreeMemory(interface_info_list.cast());

    Ok(interface_info)
  }
}

fn get_available_network_list(w_handle: &HANDLE, interface: &WLAN_INTERFACE_INFO) -> Result<Vec<WLAN_AVAILABLE_NETWORK>, WlanHandlerError> {
  unsafe {
    let mut available_network_list = std::ptr::null_mut();
    let get_available_network_list_result = WIN32_ERROR(WlanGetAvailableNetworkList(
      *w_handle,
      &interface.InterfaceGuid,
      0,
      None,
      &mut available_network_list,
    ));
    if get_available_network_list_result != ERROR_SUCCESS {
      return Err(WlanHandlerError::new(get_available_network_list_result));
    }

    // https://stackoverflow.com/a/78779478/9879620
    let networks_ptr = std::ptr::addr_of!((*available_network_list).Network);
    let networks_len = (*available_network_list).dwNumberOfItems as usize;
    let networks = std::slice::from_raw_parts(networks_ptr.cast::<WLAN_AVAILABLE_NETWORK>(), networks_len).to_vec();

    WlanFreeMemory(available_network_list.cast());

    Ok(networks)
  }
}

fn get_network_bss_list(
  w_handle: &HANDLE,
  interface: &WLAN_INTERFACE_INFO,
  network: &WLAN_AVAILABLE_NETWORK,
) -> Result<Vec<WLAN_BSS_ENTRY>, WlanHandlerError> {
  unsafe {
    let mut bssid_list = std::ptr::null_mut();
    let get_network_bss_list_result = WIN32_ERROR(WlanGetNetworkBssList(
      *w_handle,
      &interface.InterfaceGuid,
      Some(std::ptr::addr_of!(network.dot11Ssid)),
      network.dot11BssType,
      network.bSecurityEnabled,
      None,
      &mut bssid_list,
    ));
    if get_network_bss_list_result != ERROR_SUCCESS {
      return Err(WlanHandlerError::new(get_network_bss_list_result));
    }

    // https://stackoverflow.com/a/78779478/9879620
    let bss_entries_ptr = std::ptr::addr_of!((*bssid_list).wlanBssEntries);
    let bss_entries_len = (*bssid_list).dwNumberOfItems as usize;
    let bss_entries = std::slice::from_raw_parts(bss_entries_ptr.cast::<WLAN_BSS_ENTRY>(), bss_entries_len).to_vec();

    WlanFreeMemory(bssid_list.cast());

    Ok(bss_entries)
  }
}

#[allow(dead_code)]
pub fn get_available_networks() -> Result<Vec<WLAN>, WlanHandlerError> {
  let mut network_list: Vec<WLAN> = Vec::new();

  let handle = open_handle()?;

  for interface in enum_interfaces(&handle)? {
    let get_available_network_list = get_available_network_list(&handle, &interface);
    if get_available_network_list.is_err() {
      continue;
    }

    for network in get_available_network_list.unwrap() {
      network_list.push(WLAN::new(&network, get_network_bss_list(&handle, &interface, &network)?));
    }
  }

  unsafe { WlanCloseHandle(handle, None) };

  Ok(network_list)
}

#[allow(dead_code)]
pub fn is_ethernet_plugged_in() -> bool {
  let mut is_plugged_in = false;

  unsafe {
    // https://docs.microsoft.com/en-us/windows/desktop/api/iphlpapi/nf-iphlpapi-getadaptersaddresses
    let mut buf_len: core::ffi::c_ulong = 16384;
    let mut adapters_addresses_buffer = Vec::with_capacity(buf_len as usize);

    let result = WIN32_ERROR(GetAdaptersAddresses(
      0, // AF_UNSPEC
      GAA_FLAG_SKIP_UNICAST | GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST | GAA_FLAG_SKIP_DNS_SERVER,
      None,
      Some(adapters_addresses_buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH),
      &mut buf_len as *mut core::ffi::c_ulong,
    ));

    if result == ERROR_SUCCESS {
      let mut adapter_addresses_ptr = adapters_addresses_buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH;

      while !adapter_addresses_ptr.is_null() {
        let adapter = adapter_addresses_ptr.read_unaligned();

        if adapter.IfType == 6 && adapter.FriendlyName.to_string().unwrap() == "Ethernet" && adapter.OperStatus == IfOperStatusUp {
          is_plugged_in = true;
        }
        adapter_addresses_ptr = adapter.Next;
      }

      return is_plugged_in;
    } else {
      println!("Unable to get adapter addresses! {}", result.0);
      return false;
    }
  }
}

#[allow(dead_code)]
pub fn set_wifi_state(on: bool) -> Result<(), anyhow::Error> {
  let a = Radios::Radio::GetRadiosAsync()?.get()?;
  for radio in 0..a.Size()? {
    let radio = a.GetAt(radio)?;

    if radio.Kind()? == RadioKind::WiFi {
      match on {
        true => radio.SetStateAsync(Radios::RadioState::On)?,
        false => radio.SetStateAsync(Radios::RadioState::Off)?,
      };
    }
  }
  Ok(())
}
