mod mods;

use mods::media::{
  change_default_output, enumerate_audio_devices, get_active_audio_applications,
  get_default_device, init,
  types::{device::DeviceType, error::AudioDeviceError},
};
use std::{thread::sleep, time::Duration};

fn main() -> Result<(), AudioDeviceError> {
  println!("Running PCM-Backend-Test");

  // Initialize the audio device
  init()?;

  let mut connected = false;
  let discord_executable = String::from("Discord.exe");

  loop {
    // Get all output devices
    let all_outputs = enumerate_audio_devices(&DeviceType::Output)?;

    // Check if there are multiple output devices
    if all_outputs.len() > 1 {
      // Get the current default output device
      let current_output = get_default_device(&DeviceType::Output)?;

      // Check if Discord is running and recording from default input device
      let programs = get_active_audio_applications(&DeviceType::Input)?;

      if programs.contains(&discord_executable) {
        connected = true;

        // Switch to headphones if Discord is recording and speakers are the default
        if current_output.device_type == "Speakers" {
          let headphones = all_outputs
            .iter()
            .find(|device| device.device_type == "Headphones")
            .unwrap();

          change_default_output(headphones.device_id)?
        }
      } else if connected {
        connected = false;

        // Switch back to speakers if Discord is not recording and headphones are the default
        if current_output.device_type == "Headphones" {
          let headphones = all_outputs
            .iter()
            .find(|device| device.device_type == "Speakers")
            .unwrap();

          change_default_output(headphones.device_id)?
        }
      }
    }

    sleep(Duration::from_secs(1));
  }
}
