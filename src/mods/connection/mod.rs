pub(crate) mod types;

use types::{error::WlanHandlerError, network::Network};
use windows::Win32::{
  Foundation::{ERROR_SUCCESS, HANDLE, WIN32_ERROR},
  NetworkManagement::WiFi::{
    WlanCloseHandle, WlanEnumInterfaces, WlanGetAvailableNetworkList, WlanGetNetworkBssList,
    WlanOpenHandle, WLAN_AVAILABLE_NETWORK, WLAN_BSS_ENTRY, WLAN_INTERFACE_INFO,
  },
};

fn open_handle() -> Result<HANDLE, WlanHandlerError> {
  let mut w_handle = HANDLE::default();
  let mut current_version = 0;

  let open_handle_result =
    WIN32_ERROR(unsafe { WlanOpenHandle(2, None, &mut current_version, &mut w_handle) });
  if open_handle_result != ERROR_SUCCESS {
    return Err(WlanHandlerError::new(open_handle_result));
  }
  Ok(w_handle)
}

fn enum_interfaces(w_handle: &HANDLE) -> Result<Vec<WLAN_INTERFACE_INFO>, WlanHandlerError> {
  let mut interface_info_list = std::ptr::null_mut();
  let enum_interfaces_result =
    WIN32_ERROR(unsafe { WlanEnumInterfaces(*w_handle, None, &mut interface_info_list) });
  if enum_interfaces_result != ERROR_SUCCESS {
    return Err(WlanHandlerError::new(enum_interfaces_result));
  }

  Ok(
    unsafe {
      std::slice::from_raw_parts(
        std::ptr::addr_of!((*interface_info_list).InterfaceInfo).cast::<WLAN_INTERFACE_INFO>(),
        (*interface_info_list).dwNumberOfItems as usize,
      )
    }
    .to_vec(),
  )
}

fn get_available_network_list(
  w_handle: &HANDLE,
  interface: &WLAN_INTERFACE_INFO,
) -> Result<Vec<WLAN_AVAILABLE_NETWORK>, WlanHandlerError> {
  let mut available_network_list = unsafe { std::mem::zeroed() };
  let get_available_network_list_result = WIN32_ERROR(unsafe {
    WlanGetAvailableNetworkList(
      *w_handle,
      &interface.InterfaceGuid,
      0,
      None,
      &mut available_network_list,
    )
  });
  if get_available_network_list_result != ERROR_SUCCESS {
    return Err(WlanHandlerError::new(get_available_network_list_result));
  }

  Ok(
    unsafe {
      std::slice::from_raw_parts(
        std::ptr::addr_of!((*available_network_list).Network).cast::<WLAN_AVAILABLE_NETWORK>(),
        (*available_network_list).dwNumberOfItems as usize,
      )
    }
    .to_vec(),
  )
}

fn get_network_bss_list(
  w_handle: &HANDLE,
  interface: &WLAN_INTERFACE_INFO,
  network: &WLAN_AVAILABLE_NETWORK,
) -> Result<Vec<WLAN_BSS_ENTRY>, WlanHandlerError> {
  let mut bssid_list = unsafe { std::mem::zeroed() };
  let get_network_bss_list_result = WIN32_ERROR(unsafe {
    WlanGetNetworkBssList(
      *w_handle,
      &interface.InterfaceGuid,
      Some(std::ptr::addr_of!(network.dot11Ssid)),
      network.dot11BssType,
      network.bSecurityEnabled,
      None,
      &mut bssid_list,
    )
  });
  if get_network_bss_list_result != ERROR_SUCCESS {
    return Err(WlanHandlerError::new(get_network_bss_list_result));
  }

  Ok(
    unsafe {
      std::slice::from_raw_parts(
        std::ptr::addr_of!((*bssid_list).wlanBssEntries).cast::<WLAN_BSS_ENTRY>(),
        (*bssid_list).dwNumberOfItems as usize,
      )
    }
    .to_vec(),
  )
}

#[allow(dead_code)]
pub fn get_available_networks() -> Result<Vec<Network>, WlanHandlerError> {
  let mut network_list: Vec<Network> = Vec::new();

  let handle = open_handle()?;

  for interface in enum_interfaces(&handle)? {
    let get_available_network_list = get_available_network_list(&handle, &interface);
    if get_available_network_list.is_err() {
      continue;
    }

    for network in get_available_network_list.unwrap() {
      network_list.push(Network::new(
        &network,
        get_network_bss_list(&handle, &interface, &network)?,
      ));
    }
  }

  unsafe { WlanCloseHandle(handle, None) };

  return Ok(network_list);
}
