#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StartupGroup {
  User,
  System,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartupState {
  pub group: StartupGroup,
  pub path: String,
  pub name: String,
  pub status: bool,
}
