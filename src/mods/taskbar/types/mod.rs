pub struct TaskbarSize {
  pub height: u32,
  pub width: u32,
}

impl TaskbarSize {
  pub fn new() -> Self {
    Self {
      height: 0,
      width: 0,
    }
  }
}

impl Default for TaskbarSize {
  fn default() -> Self {
    Self::new()
  }
}
