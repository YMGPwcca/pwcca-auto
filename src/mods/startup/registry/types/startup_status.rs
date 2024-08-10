#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartupKind {
  User,
  System,
}

#[derive(Debug)]
pub struct StartupState {
  pub kind: StartupKind,
  pub name: String,
  pub status: bool,
}
