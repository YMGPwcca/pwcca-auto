#![allow(dead_code)]

use anyhow::Result;
use windows::Win32::{
  Foundation::{CloseHandle, HMODULE, MAX_PATH},
  System::{
    ProcessStatus::{EnumProcessModulesEx, EnumProcesses, GetModuleBaseNameW, LIST_MODULES_ALL},
    Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
  },
};

pub fn get_processes_exec_name() -> Result<Vec<String>> {
  Ok(
    get_processes()?
      .iter()
      .map(get_process_executable_name)
      .filter(|p_name| !p_name.is_empty())
      .collect(),
  )
}

fn get_processes() -> Result<Vec<u32>> {
  let mut pids = [0; 2048];
  let mut size = 0;

  unsafe { EnumProcesses(pids.as_mut_ptr(), 2048, &mut size)? };

  Ok(pids[0..(size / 4) as usize].to_vec())
}

fn get_process_executable_name(pid: &u32) -> String {
  unsafe {
    let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, *pid);
    if handle.is_ok() {
      let handle = handle.ok().unwrap();

      let mut module = HMODULE::default();
      let mut size = 0;
      let result = EnumProcessModulesEx(
        handle,
        &mut module,
        std::mem::size_of::<HMODULE>() as u32,
        &mut size,
        LIST_MODULES_ALL,
      );
      if result.is_ok() {
        let mut lpbasename = [0u16; MAX_PATH as usize];
        GetModuleBaseNameW(handle, module, &mut lpbasename);

        return String::from_utf16_lossy(&lpbasename)
          .to_lowercase()
          .split(".exe")
          .next()
          .unwrap()
          .to_string();
      }

      let _ = CloseHandle(handle);
    }
  };

  String::new()
}

pub fn get_processes_by_name(name: &str) -> Result<Vec<String>> {
  let pids = get_processes()?;

  Ok(
    pids
      .iter()
      .map(get_process_executable_name)
      .filter(|p_name| p_name == name)
      .filter(|p_name| !p_name.is_empty())
      .collect(),
  )
}
