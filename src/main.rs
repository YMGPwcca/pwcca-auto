mod mods;

use mods::media::{change_default_output, get_all_outputs, get_default_output, init, list_all_programs, Device};
use std::{thread::sleep, time::Duration};

fn main() {
  println!("Running PCM-Backend-Test");

  init();
  let mut connected = false;

  loop {
    let current_output = get_default_output();
    let all_outputs = get_all_outputs();

    if all_outputs.len() < 2 {
      continue;
    }

    if list_all_programs(&mods::media::DeviceType::Input).contains(&"Discord.exe".to_owned()) {
      connected = true;

      if current_output.device_type == "Speakers" {
        let headphones_only = all_outputs
          .into_iter()
          .filter(|device| device.device_type == "Headphones")
          .collect::<Vec<Device>>();

        change_default_output(headphones_only.first().unwrap().device_id);
      }
    } else if connected {
      connected = false;

      if current_output.device_type == "Headphones" {
        let speakers_only = all_outputs
          .into_iter()
          .filter(|device| device.device_type == "Speakers")
          .collect::<Vec<Device>>();

        change_default_output(speakers_only.first().unwrap().device_id);
      }
    }

    sleep(Duration::from_secs(1))
  }
}
