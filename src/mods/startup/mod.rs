#![allow(dead_code)]

use std::os::windows::ffi::OsStringExt;

use windows::{
  core::{Interface, BSTR, PWSTR},
  Win32::{
    Foundation::VARIANT_TRUE,
    System::{
      Com::{CoCreateInstance, CoInitialize, CoUninitialize, CLSCTX_ALL},
      TaskScheduler::{
        IExecAction, ILogonTrigger, ITaskFolder, ITaskService, TaskScheduler, TASK_ACTION_EXEC,
        TASK_CREATE_OR_UPDATE, TASK_LOGON_INTERACTIVE_TOKEN, TASK_TRIGGER_LOGON,
      },
      WindowsProgramming::GetUserNameW,
    },
  },
};

fn init_com() -> Result<(), anyhow::Error> {
  let result = unsafe { CoInitialize(None) };
  if result.is_err() {
    unsafe { CoUninitialize() };
    return Err(anyhow::Error::msg(result.message()));
  }

  Ok(())
}

fn release_com() {
  unsafe { CoUninitialize() };
}

fn get_task_service() -> Result<ITaskService, anyhow::Error> {
  init_com()?;

  unsafe {
    let task_service: ITaskService = CoCreateInstance(&TaskScheduler, None, CLSCTX_ALL)?;
    task_service.Connect(None, None, None, None)?;
    release_com();

    Ok(task_service)
  }
}

fn get_current_user() -> Result<String, anyhow::Error> {
  let mut size = 256;
  let mut buffer: Vec<u16> = vec![0; size as usize];

  unsafe { GetUserNameW(PWSTR(buffer.as_mut_ptr()), &mut size) }?;

  Ok(
    std::ffi::OsString::from_wide(&buffer[..(size - 1) as usize])
      .to_string_lossy()
      .to_string(),
  )
}

pub fn create_startup_task() -> Result<(), anyhow::Error> {
  let current_user = get_current_user()?;

  let exe_path = std::env::current_exe()?;
  let exe_dir = exe_path.parent().unwrap();

  let service = get_task_service()?;

  unsafe {
    let definition = service.NewTask(0)?;

    let settings = definition.Settings()?;
    settings.SetStartWhenAvailable(VARIANT_TRUE)?;
    settings.SetExecutionTimeLimit(&BSTR::from("PT0S"))?;
    settings.SetPriority(4)?;

    let action: IExecAction = definition.Actions()?.Create(TASK_ACTION_EXEC)?.cast()?;
    action.SetPath(&BSTR::from(exe_path.to_string_lossy().to_string()))?;
    action.SetWorkingDirectory(&BSTR::from(exe_dir.to_string_lossy().to_string()))?;

    let trigger: ILogonTrigger = definition.Triggers()?.Create(TASK_TRIGGER_LOGON)?.cast()?;
    trigger.SetUserId(&BSTR::from(&current_user))?;

    let reg_info = definition.RegistrationInfo()?;
    reg_info.SetAuthor(&BSTR::from(&current_user))?;
    reg_info.SetDescription(&BSTR::from("Run with Windows"))?;

    let folder: ITaskFolder = service.GetFolder(&BSTR::from("\\"))?;
    folder.RegisterTaskDefinition(
      &BSTR::from("PwccaAuto"),
      &definition,
      TASK_CREATE_OR_UPDATE.0,
      None,
      None,
      TASK_LOGON_INTERACTIVE_TOKEN,
      None,
    )?;

    drop(definition);
    drop(settings);
    drop(action);
    drop(trigger);
    drop(reg_info);
    drop(folder);
  };

  drop(current_user);
  drop(service);

  Ok(())
}

pub fn delete_startup_task() -> Result<(), anyhow::Error> {
  let service = get_task_service()?;

  unsafe {
    let folder = service.GetFolder(&BSTR::from("\\"))?;
    folder.DeleteTask(&BSTR::from("PwccaAuto"), 0)?;

    drop(folder);
  }

  drop(service);

  Ok(())
}
