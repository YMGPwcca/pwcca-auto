#![allow(dead_code)]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartupGroup {
  User,
  System,
}

#[derive(Debug)]
pub struct StartupState {
  pub group: StartupGroup,
  pub path: String,
  pub name: String,
  pub status: bool,
}
