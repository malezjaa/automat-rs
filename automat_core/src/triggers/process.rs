use crate::triggers::context::{TriggerContext, TriggerEvent, send_error};
use crate::{Result, Trigger, callback};
use async_trait::async_trait;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::time::Duration;
use sysinfo::{ProcessesToUpdate, System};
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;

static SYSTEM: Lazy<Mutex<System>> = Lazy::new(|| Mutex::new(System::new()));

/// Information about a process.
#[derive(Clone, Debug)]
pub struct ProcessInfo {
  pub pid: u32,
  pub name: String,
}

/// Events that can occur related to processes.
#[derive(Debug)]
pub enum ProcessEvent {
  Started(ProcessInfo),
  Exited(ProcessInfo),
}

callback!(ProcessCallback<T>);

pub struct ProcessTrigger {
  callback: ProcessCallback<TriggerContext<ProcessEvent>>,
  known_processes: HashMap<u32, String>,
  poll_interval: Duration,
}

impl ProcessTrigger {
  pub fn new<F>(f: F) -> Self
  where
    F: Fn(TriggerContext<ProcessEvent>) -> Result<()> + Send + Sync + 'static,
  {
    Self::with_interval(f, Duration::from_millis(500))
  }

  pub fn with_interval<F>(f: F, poll_interval: Duration) -> Self
  where
    F: Fn(TriggerContext<ProcessEvent>) -> Result<()> + Send + Sync + 'static,
  {
    Self {
      callback: new_process_callback(f),
      known_processes: HashMap::new(),
      poll_interval,
    }
  }

  fn refresh_and_get_processes() -> HashMap<u32, String> {
    let mut sys = SYSTEM.lock();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    sys
      .processes()
      .iter()
      .map(|(pid, proc)| (pid.as_u32(), proc.name().to_string_lossy().to_string()))
      .collect()
  }

  /// Handle process changes between current and previous state.
  async fn handle_process_changes(
    &self,
    current_processes: &HashMap<u32, String>,
    tx: &Sender<TriggerEvent>,
  ) {
    // Check for new processes
    for (pid, name) in current_processes {
      if !self.known_processes.contains_key(pid) {
        if let Err(err) = (self.callback)(TriggerContext::new(
          ProcessEvent::Started(ProcessInfo {
            pid: *pid,
            name: name.clone(),
          }),
          tx.clone(),
        )) {
          if !send_error(tx, err, "ProcessTrigger").await {
            return;
          }
        }
      }
    }

    // Check for exited processes
    for (pid, name) in &self.known_processes {
      if !current_processes.contains_key(pid) {
        if let Err(err) = (self.callback)(TriggerContext::new(
          ProcessEvent::Exited(ProcessInfo {
            pid: *pid,
            name: name.clone(),
          }),
          tx.clone(),
        )) {
          if !send_error(tx, err, "ProcessTrigger").await {
            return;
          }
        }
      }
    }
  }
}

#[async_trait]
impl Trigger for ProcessTrigger {
  async fn start(&mut self, tx: Sender<TriggerEvent>) {
    self.known_processes = Self::refresh_and_get_processes();

    loop {
      let current_processes = Self::refresh_and_get_processes();
      self.handle_process_changes(&current_processes, &tx).await;
      self.known_processes = current_processes;
      sleep(self.poll_interval).await;
    }
  }

  fn name(&self) -> String {
    "ProcessTrigger".to_string()
  }
}

pub fn get_process_name(pid: u32) -> Option<String> {
  SYSTEM
    .lock()
    .process(sysinfo::Pid::from_u32(pid))
    .map(|p| p.name().to_string_lossy().to_string())
}
