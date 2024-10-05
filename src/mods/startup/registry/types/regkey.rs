#![allow(dead_code)]

use anyhow::Result;
use windows::{
  core::{HSTRING, PCWSTR, PWSTR},
  Win32::{
    Foundation::ERROR_SUCCESS,
    System::Registry::{
      RegCloseKey, RegEnumKeyW, RegEnumValueW, RegGetValueW, RegOpenKeyExW, RegSetValueExW, HKEY,
      KEY_READ, KEY_WRITE, REG_BINARY, REG_VALUE_TYPE, RRF_RT_ANY,
    },
  },
};

#[derive(Debug)]
pub struct RegKey {
  hkey: HKEY,
  root: HKEY,
  subkey: String,
}
impl RegKey {
  pub fn open(root: HKEY, subkey: PCWSTR) -> Result<Self> {
    let mut hkey = HKEY::default();

    unsafe {
      let result = RegOpenKeyExW(root, subkey, 0, KEY_READ | KEY_WRITE, &mut hkey);

      if result == ERROR_SUCCESS {
        Ok(Self {
          hkey,
          root,
          subkey: subkey.to_string()?,
        })
      } else {
        Err(anyhow::Error::msg(result.to_hresult().message()))
      }
    }
  }

  pub fn enum_key(&self) -> Vec<String> {
    // https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-element-size-limits
    let mut index = 0;
    let mut lpname: [u16; 256] = [0; 256];
    let mut sub_keys: Vec<String> = Vec::new();

    unsafe {
      while RegEnumKeyW(self.hkey, index, Some(&mut lpname)) == ERROR_SUCCESS {
        sub_keys.push(
          String::from_utf16_lossy(&lpname)
            .trim_matches(char::from(0))
            .to_string(),
        );

        index += 1;
        lpname = [0; 256]; // Reset the value
      }
    };

    sub_keys
  }

  pub fn enum_value(&self) -> Vec<String> {
    // https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-element-size-limits
    let mut index = 0;
    let mut lpname: [u16; 256] = [0; 256];
    let mut size = 256;

    let mut values: Vec<String> = Vec::new();

    unsafe {
      while RegEnumValueW(
        self.hkey,
        index,
        PWSTR::from_raw(lpname.as_mut_ptr()),
        &mut size,
        None,
        None,
        None,
        None,
      ) == ERROR_SUCCESS
      {
        values.push(
          String::from_utf16_lossy(&lpname)
            .trim_matches(char::from(0))
            .to_string(),
        );

        index += 1;

        // Reset the value
        size = 255;
        lpname.fill(0);
      }
    }

    values
  }

  pub fn set_value_data(&self, name: &str, value: bool) {
    let value = if value { [2] } else { [3] };

    unsafe {
      let result = RegSetValueExW(self.hkey, &HSTRING::from(name), 0, REG_BINARY, Some(&value));

      if result != ERROR_SUCCESS {
        println!("Error setting value: {}", result.to_hresult().message());
      }
    }
  }

  pub fn get_value_data(&self, name: &str) -> Result<(String, Option<String>)> {
    let mut size = 2048;
    let mut buffer: [u16; 2048] = [0; 2048];

    unsafe {
      let result = RegGetValueW(
        self.hkey,
        None,
        &HSTRING::from(name),
        RRF_RT_ANY,
        None,
        Some(std::ptr::addr_of_mut!(buffer) as _),
        Some(std::ptr::addr_of_mut!(size) as _),
      );

      if result != ERROR_SUCCESS {
        println!("Error getting value: {}", result.to_hresult().message());
      }
    }

    let data = String::from_utf16_lossy(&buffer)
      .trim_matches(char::from(0))
      .replace("\"", "")
      .to_string();

    let split = data.split(" ").collect::<Vec<&str>>();
    let mut length = split.len() - 1;
    loop {
      let maybe_path = split[..length].join(" ");
      if std::path::Path::new(&maybe_path).exists() {
        let maybe_arg = data.split_at(maybe_path.len()).1.to_string();
        return Ok((maybe_path, Some(maybe_arg)));
      }
      length -= 1;
    }
  }

  pub fn is_startup_enabled(&self, value: PCWSTR) -> Result<bool> {
    let mut buffer: [u32; 512] = [0; 512];
    let mut size = (1024 * std::mem::size_of_val(&buffer[0])) as u32;
    let mut kind = REG_VALUE_TYPE::default();

    unsafe {
      let result = RegGetValueW(
        self.hkey,
        None,
        value,
        RRF_RT_ANY,
        Some(&mut kind),
        Some(buffer.as_mut_ptr() as _),
        Some(&mut size),
      );

      if result == ERROR_SUCCESS {
        if (&buffer[..size as usize])[0] == 3 {
          return Ok(false);
        }
        if (&buffer[..size as usize])[0] == 2 {
          return Ok(true);
        }
      }

      Err(anyhow::Error::msg(result.to_hresult().message()))
    }
  }
}

impl Drop for RegKey {
  fn drop(&mut self) {
    unsafe {
      let _ = RegCloseKey(self.hkey);
    }
  }
}
