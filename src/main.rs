mod mods;

use mods::media::{change_default_output, get_all_outputs, get_default_output, init, list_all_programs, Device};
use std::{thread::sleep, time::Duration};

fn main() {
  init();
  let mut disconnect = false;

  loop {  
    let current_output = get_default_output();

    if list_all_programs(&mods::media::DeviceType::Input).contains(&"Discord.exe".to_owned()) {
      disconnect = false;
      
      if current_output.device_type == "Speakers" {
        let outputs = get_all_outputs();
        let headphones_only = outputs
          .into_iter()
          .filter(|device| device.device_type == "Headphones")
          .collect::<Vec<Device>>();

        change_default_output(headphones_only.first().unwrap().device_id);
      }
    } else {
      if !disconnect {
        disconnect = true;

        if current_output.device_type == "Headphones" {
          let outputs = get_all_outputs();
          let speakers_only = outputs
            .into_iter()
            .filter(|device| device.device_type == "Speakers")
            .collect::<Vec<Device>>();

          change_default_output(speakers_only.first().unwrap().device_id);
        }
      }
    }

    sleep(Duration::from_secs(1))
  }
}
