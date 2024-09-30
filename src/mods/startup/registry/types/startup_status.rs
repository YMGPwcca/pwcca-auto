#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StartupGroup {
  User,
  System,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StartupKind {
  Registry,
  Folder,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartupItem {
  pub kind: StartupKind,
  pub group: StartupGroup,
  pub path: String,
  pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartupState {
  pub kind: StartupKind,
  pub group: StartupGroup,
  pub path: String,
  pub state_path: String,
  pub name: String,
  pub state: bool,
}
