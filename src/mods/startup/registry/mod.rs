#![allow(dead_code)]

pub mod types;

use std::fs;

use anyhow::Result;
use types::{
  regkey::RegKey,
  startup_status::{StartupGroup, StartupItem, StartupKind, StartupState},
};
use windows::{
  core::{HSTRING, PCWSTR},
  Win32::System::Registry::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE},
};

pub fn get_all_startup_items() -> Result<Vec<StartupState>> {
  let mut items = get_startup_items_by_group(StartupGroup::User)?;
  items.append(&mut get_startup_items_by_group(StartupGroup::System)?);

  Ok(items)
}

pub fn get_startup_items_by_group(group: StartupGroup) -> Result<Vec<StartupState>> {
  let mut items = get_startup_items_in_registry(&group)?;
  items.append(&mut get_startup_items_in_folder(&group)?);

  let states = get_startup_item_state(&group, &items)?;

  Ok(states)
}

pub fn get_startup_item_value(item: &StartupState) -> Result<String> {
  let root = match item.group {
    StartupGroup::User => HKEY_CURRENT_USER,
    StartupGroup::System => HKEY_LOCAL_MACHINE,
  };

  if item.kind == StartupKind::Registry {
    let key = RegKey::open(root, PCWSTR(HSTRING::from(&item.path).as_ptr()))?;
    return key.get_value_data(&item.name);
  }

  Ok(item.path.clone())
}

fn get_startup_items_in_registry(group: &StartupGroup) -> Result<Vec<StartupItem>> {
  let mut items = Vec::new();

  let root = match group {
    StartupGroup::User => HKEY_CURRENT_USER,
    StartupGroup::System => HKEY_LOCAL_MACHINE,
  };

  let paths = match group {
    StartupGroup::User => vec![
      String::from(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run\"), // Run
    ],
    StartupGroup::System => vec![
      String::from(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run\"), // Run
      String::from(r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Run\"), // Run32
    ],
  };

  for path in paths {
    let key = RegKey::open(root, PCWSTR(HSTRING::from(&path).as_ptr()))?;
    for value in key.enum_value() {
      items.push(StartupItem {
        kind: StartupKind::Registry,
        group: *group,
        path: path.clone(),
        name: value,
      });
    }
  }

  Ok(items)
}

fn get_startup_items_in_folder(group: &StartupGroup) -> Result<Vec<StartupItem>> {
  let mut items = Vec::new();

  let dir_path = match group {
    StartupGroup::User => std::env::var("APPDATA")?,
    StartupGroup::System => std::env::var("PROGRAMDATA")?,
  } + r"\Microsoft\Windows\Start Menu\Programs\Startup";

  let dir_items = fs::read_dir(dir_path)?;

  for item in dir_items {
    let item = item?;
    items.push(StartupItem {
      kind: StartupKind::Folder,
      group: *group,
      path: item.path().parent().unwrap().to_string_lossy().to_string(),
      name: item.file_name().to_string_lossy().to_string(),
    });
  }

  Ok(items)
}

fn get_startup_item_state(
  group: &StartupGroup,
  items: &[StartupItem],
) -> Result<Vec<StartupState>> {
  let mut result: Vec<StartupState> = Vec::new();

  let root = match group {
    StartupGroup::User => HKEY_CURRENT_USER,
    StartupGroup::System => HKEY_LOCAL_MACHINE,
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

      let contain = items
        .iter()
        .map(|e| e.name.clone())
        .position(|e| e == value);
      if contain.is_some() {
        let item = &items[contain.unwrap()];

        result.push(StartupState {
          kind: item.kind,
          group: item.group,
          path: item.path.clone(),
          state_path: approved_path.clone() + &key,
          name: String::from(&value),
          state: data,
        });
      }
    }
  }

  Ok(result)
}

pub fn set_startup_item_state(item: &StartupState, status: bool) -> Result<()> {
  let root = match item.group {
    StartupGroup::User => HKEY_CURRENT_USER,
    StartupGroup::System => HKEY_LOCAL_MACHINE,
  };

  let key = RegKey::open(root, PCWSTR(HSTRING::from(&item.state_path).as_ptr()))?;

  key.set_value_data(&item.name, status);

  Ok(())
}
