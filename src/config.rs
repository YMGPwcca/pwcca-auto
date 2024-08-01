// Mew was here
#![allow(dead_code)]

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Config {
  pub discord: bool,
  pub ethernet: bool,
  pub power: Power,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Power {
  pub timer: u32,
  pub percentage: u32,
}

impl Config {
  pub const fn new() -> Self {
    Config {
      discord: false,
      ethernet: false,
      power: Power {
        timer: 300,
        percentage: 60,
      },
    }
  }

  fn get_path() -> Result<std::path::PathBuf> {
    let exe_path = std::env::current_exe()?;
    let config_path = std::path::Path::new(exe_path.parent().unwrap()).join("config.json");
    Ok(config_path)
  }

  pub fn toggle_discord(&mut self) {
    self.discord = !self.discord;
  }

  pub fn toggle_ethernet(&mut self) {
    self.ethernet = !self.ethernet;
  }

  pub fn set_power(&mut self, timer: u32, percentage: u32) {
    self.power = Power { timer, percentage };
  }

  pub fn write(&self) -> Result<Self> {
    std::fs::write(Config::get_path()?, self.stringify()?)?;
    Ok(*self)
  }

  pub fn read() -> Result<Self> {
    let path = Config::get_path()?;
    if path.exists() {
      Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
    } else {
      Ok(Config::write(&Config::new())?)
    }
  }

  pub fn stringify(&self) -> Result<String> {
    Ok(serde_json::to_string_pretty(self)?)
  }
}
