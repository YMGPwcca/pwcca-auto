pub mod types;

use std::fs;

use anyhow::Result;
use types::{
  regkey::RegKey,
  startup_status::{StartupKind, StartupState},
};
use windows::{
  core::{HSTRING, PCWSTR},
  Win32::System::Registry::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE},
};

pub fn get_startup_items(kind: StartupKind) -> Result<Vec<StartupState>> {
  let mut items = get_startup_items_in_registry(&kind)?;
  items.append(&mut get_startup_items_in_folder(&kind)?);

  let states = get_startup_item_state(&kind, &items)?;

  Ok(states)
}

fn get_startup_items_in_registry(kind: &StartupKind) -> Result<Vec<String>> {
  let mut items = Vec::new();

  let root = match kind {
    StartupKind::User => HKEY_CURRENT_USER,
    StartupKind::System => HKEY_LOCAL_MACHINE,
  };

  let paths = match kind {
    StartupKind::User => vec![
      String::from(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run\"), // Run
    ],
    StartupKind::System => vec![
      String::from(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run\"), // Run
      String::from(r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Run\"), // Run32
    ],
  };

  for path in paths {
    let key = RegKey::open(root, PCWSTR(HSTRING::from(&path).as_ptr()))?;
    items.append(&mut key.enum_value());
  }

  Ok(items)
}

fn get_startup_items_in_folder(kind: &StartupKind) -> Result<Vec<String>> {
  let mut items = Vec::new();

  let dir_path = match kind {
    StartupKind::User => std::env::var("APPDATA")?,
    StartupKind::System => std::env::var("PROGRAMDATA")?,
  } + r"\Microsoft\Windows\Start Menu\Programs\Startup";

  let dir_items = fs::read_dir(dir_path)?;

  for item in dir_items {
    items.push(item?.file_name().to_string_lossy().to_string());
  }

  Ok(items)
}

fn get_startup_item_state(kind: &StartupKind, items: &[String]) -> Result<Vec<StartupState>> {
  let mut result: Vec<StartupState> = Vec::new();

  let root = match kind {
    StartupKind::User => HKEY_CURRENT_USER,
    StartupKind::System => HKEY_LOCAL_MACHINE,
  };

  let approved_path =
    String::from(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\");

  let user_key = RegKey::open(root, PCWSTR(HSTRING::from(&approved_path).as_ptr()))?;

  for key in user_key.enum_key() {
    let startup_key = RegKey::open(
      root,
      PCWSTR(HSTRING::from(approved_path.clone() + &key).as_ptr()),
    )?;

    for value in startup_key.enum_value() {
      let data = RegKey::open(
        root,
        PCWSTR(HSTRING::from(approved_path.clone() + &key).as_ptr()),
      )?
      .is_startup_enabled(PCWSTR(HSTRING::from(&value).as_ptr()))?;

      let contain = items.contains(&String::from(&value));
      if contain {
        result.push(StartupState {
          kind: *kind,
          name: String::from(&value),
          status: data,
        });
      }
    }
  }

  Ok(result)
}
