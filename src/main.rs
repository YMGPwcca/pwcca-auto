mod mods;

use std::{thread::sleep, time::Duration};
use mods::media::{change_default_output, get_all_outputs, get_current_output, is_discord_recording};

fn main() {
  loop {
    if is_discord_recording() {
      let current = get_current_output();

      if current.device_name.to_lowercase().contains("speaker") {
        let mut outputs = get_all_outputs();
        let position = outputs
          .clone()
          .into_iter()
          .position(|device| device.device_name.to_lowercase().contains("speaker"))
          .unwrap();
        outputs.remove(position);

        change_default_output(outputs.get(0).unwrap().device_id);
      }
    } else {
      let current = get_current_output();

      if current.device_name.to_lowercase().contains("headphone") {
        let mut outputs = get_all_outputs();
        let position = outputs
          .clone()
          .into_iter()
          .position(|device| device.device_name.to_lowercase().contains("headphone"))
          .unwrap();
        outputs.remove(position);

        change_default_output(outputs.get(0).unwrap().device_id);
      }

      sleep(Duration::from_secs(1))
    }
  }
}
