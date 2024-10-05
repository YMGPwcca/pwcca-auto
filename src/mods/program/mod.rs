// https://github.com/microsoft/Windows-classic-samples/blob/main/Samples/Win7Samples/winui/shell/appplatform/ExecInExplorer/ExecInExplorer.cpp
#![allow(dead_code)]

use anyhow::{Error, Result};
use windows::{
  core::{Interface, BSTR, VARIANT},
  Win32::{
    Foundation::HWND,
    System::Com::{
      CoCreateInstance, CoInitializeEx, CoUninitialize, IDispatch, CLSCTX_LOCAL_SERVER, COINIT_APARTMENTTHREADED,
      COINIT_DISABLE_OLE1DDE,
    },
    UI::Shell::{
      IShellBrowser, IShellDispatch2, IShellFolderViewDual, IShellView, IShellWindows, IUnknown_QueryService,
      SID_STopLevelBrowser, ShellWindows, SVGIO_BACKGROUND, SWC_DESKTOP, SWFO_NEEDDISPATCH,
    },
  },
};

pub struct Program {
  shell_view: IShellView,
  shell_dispatch: IShellDispatch2,
}

impl Program {
  pub fn new() -> Result<Self> {
    let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) };
    if hr.is_err() {
      return Err(Error::msg(hr.message()));
    }

    let shell_view = Program::get_shell_view()?;
    let shell_dispatch = Program::get_shell_dispatch(&shell_view)?;

    Ok(Program {
      shell_view,
      shell_dispatch,
    })
  }

  fn get_shell_view() -> Result<IShellView> {
    unsafe {
      let shell_windows: IShellWindows = CoCreateInstance(&ShellWindows, None, CLSCTX_LOCAL_SERVER)?;

      let mut hwnd = HWND::default();
      let dispatch = shell_windows.FindWindowSW(
        &VARIANT::default(),
        &VARIANT::default(),
        SWC_DESKTOP,
        std::ptr::addr_of_mut!(hwnd) as _,
        SWFO_NEEDDISPATCH,
      )?;

      let shell_browser: IShellBrowser = IUnknown_QueryService(&dispatch, &SID_STopLevelBrowser)?;
      let shell_view: IShellView = shell_browser.QueryActiveShellView()?;

      Ok(shell_view)
    }
  }

  fn get_shell_dispatch(shell_view: &IShellView) -> Result<IShellDispatch2> {
    unsafe {
      let dispatch_background: IDispatch = shell_view.GetItemObject(SVGIO_BACKGROUND)?;

      let mut p_shell_folder_view_dual = std::ptr::null_mut();
      let hr = dispatch_background.query(&IShellFolderViewDual::IID, &mut p_shell_folder_view_dual);

      if hr.is_err() {
        return Err(Error::msg(hr.message()));
      }

      let shell_folder_view_dual = IShellFolderViewDual::from_raw(p_shell_folder_view_dual);

      let dispatch = shell_folder_view_dual.Application()?;
      let mut p_shell_dispatch_2 = std::ptr::null_mut();
      let hr = dispatch.query(&IShellDispatch2::IID, &mut p_shell_dispatch_2);
      if hr.is_err() {
        return Err(Error::msg(hr.message()));
      }

      Ok(IShellDispatch2::from_raw(p_shell_dispatch_2))
    }
  }

  pub fn run(&self, file: String, args: Option<String>) -> Result<()> {
    unsafe {
      let file = BSTR::from(file);
      let args = if let Some(args) = args {
        VARIANT::from(BSTR::from(args))
      } else {
        VARIANT::default()
      };

      self.shell_dispatch.ShellExecute(
        &file,
        &args,
        &VARIANT::default(),
        &VARIANT::default(),
        &VARIANT::default(),
      )?;
    }

    Ok(())
  }
}

impl Drop for Program {
  fn drop(&mut self) {
    unsafe { CoUninitialize() };
  }
}
