#![allow(dead_code)]

pub mod types;

use types::TaskbarSize;
use windows::Win32::{
  Foundation::{BOOL, FALSE, HWND, LPARAM, RECT, TRUE},
  Graphics::Gdi::{EnumDisplaySettingsW, DEVMODEW, ENUM_CURRENT_SETTINGS},
  UI::{
    Shell::{
      SHAppBarMessage, ABM_GETSTATE, ABM_SETSTATE, ABS_ALWAYSONTOP, ABS_AUTOHIDE, APPBARDATA,
    },
    WindowsAndMessaging::{
      EnumWindows, IsWindowVisible, IsZoomed, SystemParametersInfoW, SPI_GETWORKAREA,
      SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
    },
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

pub fn get_taskbar_size() -> TaskbarSize {
  let mut workarea = RECT::default();
  let mut screen_size = DEVMODEW::default();

  let mut taskbar = TaskbarSize::default();

  unsafe {
    SystemParametersInfoW(
      SPI_GETWORKAREA,
      0,
      Some(std::ptr::addr_of_mut!(workarea) as _),
      SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
    )
    .expect("Cannot get work area size");

    EnumDisplaySettingsW(None, ENUM_CURRENT_SETTINGS, &mut screen_size)
      .expect("Cannot get screen size");
  };

  if workarea.bottom != screen_size.dmPelsHeight as i32 {
    taskbar.height = screen_size.dmPelsHeight - workarea.bottom as u32;
  }

  if workarea.right != screen_size.dmPelsWidth as i32 {
    taskbar.width = screen_size.dmPelsWidth - workarea.right as u32;
  }

  taskbar
}
