#![allow(unused)]

use windows::Win32::{
  Foundation::{CloseHandle, BOOL, HWND, LPARAM},
  UI::{
    Shell::{
      SHAppBarMessage, ABM_GETSTATE, ABM_SETSTATE, ABS_ALWAYSONTOP, ABS_AUTOHIDE, APPBARDATA,
    },
    WindowsAndMessaging::{
      EnumWindows, GetAncestor, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible, IsZoomed,
      GA_ROOT,
    },
  },
};

static mut PROGRAMS: Vec<isize> = Vec::new();

extern "system" fn enum_window(handle: HWND, lparam: LPARAM) -> BOOL {
  unsafe {
    if IsWindowVisible(handle).as_bool() && IsZoomed(handle).as_bool() {
      PROGRAMS.push(lparam.0);
      CloseHandle(handle);
      return BOOL(0);
    }

    CloseHandle(handle);
    BOOL(1)
  }
}

pub fn taskbar_automation() {
  unsafe {
    PROGRAMS.clear();
    let _ = EnumWindows(Some(enum_window), LPARAM::default());
    hide_taskbar(PROGRAMS.is_empty());
  };
}

fn hide_taskbar(hide: bool) {
  let mut pdata = APPBARDATA {
    cbSize: std::mem::size_of::<APPBARDATA>() as u32,
    ..Default::default()
  };
  let result = unsafe { SHAppBarMessage(ABM_GETSTATE, &mut pdata) };
  pdata.lParam = LPARAM(if hide { ABS_AUTOHIDE } else { ABS_ALWAYSONTOP } as isize);

  let _ = unsafe { SHAppBarMessage(ABM_SETSTATE, &mut pdata) };
}
