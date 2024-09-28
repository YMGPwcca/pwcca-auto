#![allow(dead_code)]

pub mod types;

use std::{path::Path, str::FromStr};

use types::TaskbarSize;
use windows::Win32::{
  Foundation::{CloseHandle, BOOL, FALSE, HWND, LPARAM, MAX_PATH, TRUE},
  System::{
    ProcessStatus::GetProcessImageFileNameW,
    Threading::{OpenProcess, PROCESS_ALL_ACCESS},
  },
  UI::{
    Shell::{
      SHAppBarMessage, ABM_GETSTATE, ABM_GETTASKBARPOS, ABM_SETSTATE, ABS_ALWAYSONTOP,
      ABS_AUTOHIDE, APPBARDATA,
    },
    WindowsAndMessaging::{EnumWindows, GetWindowThreadProcessId, IsWindowVisible, IsZoomed},
  },
};

use crate::CONFIG;

static mut PROGRAMS: Vec<HWND> = Vec::new();

unsafe extern "system" fn enum_window(handle: HWND, _lparam: LPARAM) -> BOOL {
  PROGRAMS = Vec::new();

  if IsWindowVisible(handle) == TRUE && IsZoomed(handle) == TRUE && !PROGRAMS.contains(&handle) {
    let mut process_id = 0;
    unsafe { GetWindowThreadProcessId(handle, Some(&mut process_id)) };

    let process_name = get_process_name(process_id);
    if CONFIG.taskbar.apps.contains(&process_name) {
      PROGRAMS.push(handle);
      return FALSE;
    }
  }

  TRUE
}

fn get_process_name(process_id: u32) -> String {
  let h_process = unsafe { OpenProcess(PROCESS_ALL_ACCESS, false, process_id) }.unwrap_or_default();

  if !h_process.is_invalid() {
    let mut process_path_buffer = [0; MAX_PATH as usize];
    let byte_written = unsafe { GetProcessImageFileNameW(h_process, &mut process_path_buffer) };

    unsafe { CloseHandle(h_process).unwrap() };
    if byte_written == 0 {
      return String::new();
    };

    let process_path =
      String::from_utf16(&process_path_buffer[..byte_written as usize]).unwrap_or_default();
    let process_name = String::from_str(
      Path::new(&process_path)
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default(),
    )
    .unwrap_or_default()
    .to_lowercase();

    return process_name;
  }

  String::new()
}

pub fn taskbar_automation() {
  let _ = unsafe { EnumWindows(Some(enum_window), LPARAM(0)) };
  hide_taskbar(unsafe { PROGRAMS.is_empty() });
}

fn hide_taskbar(hide: bool) {
  let mut pdata = APPBARDATA {
    cbSize: std::mem::size_of::<APPBARDATA>() as u32,
    ..Default::default()
  };
  unsafe { SHAppBarMessage(ABM_GETSTATE, &mut pdata) };
  pdata.lParam = LPARAM(if hide { ABS_AUTOHIDE } else { ABS_ALWAYSONTOP } as isize);

  let _ = unsafe { SHAppBarMessage(ABM_SETSTATE, &mut pdata) };
}

pub fn get_taskbar_size() -> TaskbarSize {
  let mut taskbar = TaskbarSize::default();
  let mut pdata = APPBARDATA {
    cbSize: std::mem::size_of::<APPBARDATA>() as u32,
    ..APPBARDATA::default()
  };

  unsafe { SHAppBarMessage(ABM_GETTASKBARPOS, &mut pdata) };

  taskbar.height = (pdata.rc.bottom - pdata.rc.top) as u32;
  taskbar.width = (pdata.rc.right - pdata.rc.left) as u32;

  taskbar
}
