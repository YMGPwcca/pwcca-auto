use windows::Win32::NetworkManagement::WiFi::{self, WLAN_AVAILABLE_NETWORK, WLAN_BSS_ENTRY};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Wlan {
  pub kind: String,
  pub name: String,
  pub signal_quality: u32,
  pub rssi: i32,
  pub security: String,
  pub authentication: String,
  pub flags: String,
  pub ratio_type: Vec<String>,
  pub bands: Vec<f32>,
}

#[allow(dead_code)]
impl Wlan {
  pub fn new(network: &WLAN_AVAILABLE_NETWORK, bss_entries: Vec<WLAN_BSS_ENTRY>) -> Self {
    Self {
      kind: match network.dot11BssType {
        WiFi::dot11_BSS_type_infrastructure => "Infrastructure".to_string(),
        WiFi::dot11_BSS_type_independent => "Independent".to_string(),
        WiFi::dot11_BSS_type_any => "Any".to_string(),
        _ => format!("Unknown ({:?})", network.dot11BssType),
      },

      name: String::from_utf8_lossy(&network.dot11Ssid.ucSSID[..network.dot11Ssid.uSSIDLength as usize]).to_string(),

      signal_quality: network.wlanSignalQuality,

      // https://stackoverflow.com/a/15798024/9879620
      rssi: -100 + (network.wlanSignalQuality / 2) as i32,

      // https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wlantypes/ne-wlantypes-_dot11_cipher_algorithm#constants
      security: match network.dot11DefaultCipherAlgorithm {
        WiFi::DOT11_CIPHER_ALGO_NONE => "None".to_string(),
        WiFi::DOT11_CIPHER_ALGO_WEP40 => "WEP-40".to_string(),
        WiFi::DOT11_CIPHER_ALGO_TKIP => "TKIP".to_string(),
        WiFi::DOT11_CIPHER_ALGO_CCMP => "CCMP".to_string(),
        WiFi::DOT11_CIPHER_ALGO_WEP104 => "WEP-104".to_string(),
        WiFi::DOT11_CIPHER_ALGO_BIP => "BIP".to_string(),
        WiFi::DOT11_CIPHER_ALGO_GCMP_256 => "GCMP-256".to_string(),
        WiFi::DOT11_CIPHER_ALGO_CCMP_256 => "CCMP-256".to_string(),
        WiFi::DOT11_CIPHER_ALGO_BIP_GMAC_128 => "BIP-GMAC-128".to_string(),
        WiFi::DOT11_CIPHER_ALGO_BIP_GMAC_256 => "BIP-GMAC-256".to_string(),
        WiFi::DOT11_CIPHER_ALGO_BIP_CMAC_256 => "BIP-CMAC-256".to_string(),
        WiFi::DOT11_CIPHER_ALGO_WPA_USE_GROUP => "WPA/RSN USE_GROUP".to_string(),
        WiFi::DOT11_CIPHER_ALGO_WEP => "WEP".to_string(),
        _ => format!("Unknown ({:?})", network.dot11DefaultAuthAlgorithm),
      },

      // https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wlantypes/ne-wlantypes-_dot11_auth_algorithm#constants
      authentication: match network.dot11DefaultAuthAlgorithm {
        WiFi::DOT11_AUTH_ALGO_80211_OPEN => "802.11 Open".to_string(),
        WiFi::DOT11_AUTH_ALGO_80211_SHARED_KEY => "802.11 Shared Key".to_string(),
        WiFi::DOT11_AUTH_ALGO_WPA => "WPA".to_string(),
        WiFi::DOT11_AUTH_ALGO_WPA_PSK => "WPA-PSK".to_string(),
        WiFi::DOT11_AUTH_ALGO_WPA_NONE => "WPA-None (Not Supported)".to_string(),
        WiFi::DOT11_AUTH_ALGO_RSNA => "RSNA".to_string(),
        WiFi::DOT11_AUTH_ALGO_RSNA_PSK => "WPA2-Personal".to_string(),
        WiFi::DOT11_AUTH_ALGO_WPA3 => "WPA3-Enterprise 192-bit".to_string(),
        WiFi::DOT11_AUTH_ALGO_WPA3_SAE => "WPA3-Personal".to_string(),
        WiFi::DOT11_AUTH_ALGO_OWE => "OWE".to_string(),
        WiFi::DOT11_AUTH_ALGO_WPA3_ENT => "WPA3-Enterprise".to_string(),
        _ => format!("Unknown ({:?})", network.dot11DefaultAuthAlgorithm),
      },

      flags: match network.dwFlags {
        0 => "Never Connected".to_string(),
        WiFi::WLAN_AVAILABLE_NETWORK_CONNECTED => "Connected".to_string(),
        WiFi::WLAN_AVAILABLE_NETWORK_HAS_PROFILE => "Has Profile".to_string(),
        _ => format!("Unknown ({:?})", network.dwFlags),
      },

      // https://en.wikipedia.org/wiki/IEEE_802.11
      ratio_type: Vec::from_iter(network.dot11PhyTypes[..network.uNumberOfPhyTypes as usize].iter())
        .iter()
        .map(|&e| match *e {
          WiFi::dot11_phy_type_unknown => "Unknown".to_string(),
          WiFi::dot11_phy_type_fhss => "802.11 FHSS".to_string(),
          WiFi::dot11_phy_type_dsss => "802.11 DSSS".to_string(),
          WiFi::dot11_phy_type_irbaseband => "IR Baseband".to_string(),
          WiFi::dot11_phy_type_ofdm => "802.11a".to_string(),
          WiFi::dot11_phy_type_hrdsss => "802.11b".to_string(),
          WiFi::dot11_phy_type_erp => "802.11g".to_string(),
          WiFi::dot11_phy_type_ht => "802.11n".to_string(),
          WiFi::dot11_phy_type_vht => "802.11ac".to_string(),
          WiFi::dot11_phy_type_he => "802.11ax".to_string(),
          WiFi::dot11_phy_type_eht => "802.11be".to_string(),
          _ => format!("Unknown ({:?})", e),
        })
        .collect(),

      // https://en.wikipedia.org/wiki/List_of_WLAN_channels
      bands: {
        let mut map: Vec<f32> = bss_entries
          .iter()
          .map(|&e| {
            let freq_in_mhz = format!("{:.1}", e.ulChCenterFrequency / 1_000).parse::<i32>().unwrap();
            match freq_in_mhz {
              2401..=2495 => 2.4,
              5150..=5895 => 5.0,
              5925..=7125 => 6.0,
              _ => format!("{:.1}", freq_in_mhz as f32 / 1_000.0).parse::<f32>().unwrap(),
            }
          })
          .collect();
        map.sort_by(|a, b| a.total_cmp(b));
        Vec::dedup(&mut map);
        map
      },
    }
  }
}
