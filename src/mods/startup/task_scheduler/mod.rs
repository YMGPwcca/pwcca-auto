#![allow(dead_code)]

use anyhow::Result;
use windows::{
  core::{Interface, BSTR},
  Win32::{
    Foundation::{VARIANT_FALSE, VARIANT_TRUE},
    System::{
      Com::{CoCreateInstance, CoInitialize, CoUninitialize, CLSCTX_ALL},
      TaskScheduler::{
        IExecAction, ILogonTrigger, ITaskFolder, ITaskService, TaskScheduler as GUID,
        TASK_ACTION_EXEC, TASK_CREATE_OR_UPDATE, TASK_INSTANCES_STOP_EXISTING,
        TASK_LOGON_INTERACTIVE_TOKEN, TASK_RUNLEVEL_HIGHEST, TASK_TRIGGER_LOGON,
      },
    },
  },
};

pub struct TaskScheduler(ITaskService);

impl TaskScheduler {
  pub fn new() -> Result<Self> {
    let result = unsafe { CoInitialize(None) };
    if result.is_err() {
      unsafe { CoUninitialize() };
      return Err(anyhow::Error::msg(result.message()));
    }
    let service: ITaskService = unsafe { CoCreateInstance(&GUID, None, CLSCTX_ALL) }?;
    unsafe { service.Connect(None, None, None, None) }?;

    Ok(Self(service))
  }

  pub fn create_startup_task(&self, name: &str) -> Result<()> {
    let current_user = env!("USERNAME");

    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().unwrap();

    unsafe {
      let definition = self.0.NewTask(0)?;

      let principal = definition.Principal()?;
      principal.SetRunLevel(TASK_RUNLEVEL_HIGHEST)?;

      let settings = definition.Settings()?;
      settings.SetStartWhenAvailable(VARIANT_TRUE)?;
      settings.SetExecutionTimeLimit(&BSTR::from("PT0S"))?;
      settings.SetPriority(4)?;
      settings.SetDisallowStartIfOnBatteries(VARIANT_FALSE)?;
      settings.SetStopIfGoingOnBatteries(VARIANT_FALSE)?;
      settings.SetMultipleInstances(TASK_INSTANCES_STOP_EXISTING)?;

      let action: IExecAction = definition.Actions()?.Create(TASK_ACTION_EXEC)?.cast()?;
      action.SetPath(&BSTR::from(exe_path.to_string_lossy().to_string()))?;
      action.SetWorkingDirectory(&BSTR::from(exe_dir.to_string_lossy().to_string()))?;

      let trigger: ILogonTrigger = definition.Triggers()?.Create(TASK_TRIGGER_LOGON)?.cast()?;
      trigger.SetUserId(&BSTR::from(current_user))?;
      trigger.SetDelay(&BSTR::from("PT3S"))?;

      let reg_info = definition.RegistrationInfo()?;
      reg_info.SetAuthor(&BSTR::from(current_user))?;
      reg_info.SetDescription(&BSTR::from("Run PwccaAuto with Windows"))?;

      let folder: ITaskFolder = self.0.GetFolder(&BSTR::from(r"\"))?;
      folder.RegisterTaskDefinition(
        &BSTR::from(name),
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

    Ok(())
  }

  pub fn delete_startup_task(&self, name: &str) -> Result<()> {
    unsafe {
      let folder = self.0.GetFolder(&BSTR::from(r"\"))?;
      folder.DeleteTask(&BSTR::from(name), 0)?;

      drop(folder);
    }

    Ok(())
  }

  pub fn is_service_created(&self, name: &str) -> bool {
    unsafe {
      let folder = self
        .0
        .GetFolder(&BSTR::from(r"\"))
        .expect("Cannot get folder");
      let task = folder.GetTask(&BSTR::from(name));

      if task.is_err() {
        return false;
      }
    }

    true
  }
}

impl Drop for TaskScheduler {
  fn drop(&mut self) {
    unsafe { CoUninitialize() };
  }
}
