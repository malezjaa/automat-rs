use crate::triggers::context::{TriggerContext, TriggerEvent};
use crate::{callback, pair_api, send_err, Result, Trigger, TriggerRuntime};
use async_trait::async_trait;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::time::Duration;
use derivative::Derivative;
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

#[derive(Derivative)]
#[derivative(Debug)]
pub struct ProcessTrigger {
  #[derivative(Debug = "ignore")]
  callback: ProcessCallback<TriggerContext<ProcessEvent>>,
  known_processes: HashMap<u32, String>,
  poll_interval: Duration,
}

impl ProcessTrigger {
  pair_api! {
    assoc
      new(f: F)
        callback(TriggerContext<ProcessEvent>)
        async => Self::with_interval(f, Duration::from_millis(500));
        blocking => Self::with_interval_blocking(f, Duration::from_millis(500));
  }

  pair_api! {
    assoc
      with_interval(f: F, poll_interval: Duration)
        callback(TriggerContext<ProcessEvent>)
        async => Self { callback: new_process_callback(f), known_processes: HashMap::new(), poll_interval };
        blocking => Self { callback: new_process_callback_blocking(f), known_processes: HashMap::new(), poll_interval };
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
        let context = TriggerContext::new(
          ProcessEvent::Started(ProcessInfo {
            pid: *pid,
            name: name.clone(),
          }),
          tx.clone(),
        );

        send_err!((self.callback)(context).await, "ProcessTrigger", tx, return);
      }
    }

    // Check for exited processes
    for (pid, name) in &self.known_processes {
      if !current_processes.contains_key(pid) {
        let context = TriggerContext::new(
          ProcessEvent::Exited(ProcessInfo {
            pid: *pid,
            name: name.clone(),
          }),
          tx.clone(),
        );

        send_err!((self.callback)(context).await, "ProcessTrigger", tx, return);
      }
    }
  }
}

#[async_trait]
impl Trigger for ProcessTrigger {
  async fn start(&mut self, rt: TriggerRuntime) -> Result<()> {
    self.known_processes = Self::refresh_and_get_processes();

    loop {
      if rt.shutdown.is_cancelled() {
        break;
      }
      let current_processes = Self::refresh_and_get_processes();
      self
        .handle_process_changes(&current_processes, &rt.tx)
        .await;
      self.known_processes = current_processes;

      tokio::select! {
        _ = rt.shutdown.cancelled() => break,
        _ = sleep(self.poll_interval) => {}
      }
    }

    Ok(())
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
