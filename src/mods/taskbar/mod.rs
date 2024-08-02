#![allow(dead_code)]

use windows::Win32::{
  Foundation::{BOOL, FALSE, HWND, LPARAM, TRUE},
  UI::{
    Shell::{
      SHAppBarMessage, ABM_GETSTATE, ABM_SETSTATE, ABS_ALWAYSONTOP, ABS_AUTOHIDE, APPBARDATA,
    },
    WindowsAndMessaging::{EnumWindows, IsWindowVisible, IsZoomed},
  },
};

extern "system" fn enum_window(handle: HWND, lparam: LPARAM) -> BOOL {
  unsafe {
    let programs = &mut *(lparam.0 as *mut Vec<isize>);
    if IsWindowVisible(handle) == TRUE && IsZoomed(handle) == TRUE {
      programs.push(lparam.0);
      return FALSE;
    }

    TRUE
  }
}

pub fn taskbar_automation() {
  let programs: Vec<isize> = Vec::new();

  unsafe {
    let _ = EnumWindows(
      Some(enum_window),
      LPARAM(std::ptr::addr_of!(programs) as isize),
    );
  };
  hide_taskbar(programs.is_empty());
  drop(programs);
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
