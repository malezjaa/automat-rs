use crate::{callback, Result, Trigger};
use async_trait::async_trait;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::time::Duration;
use sysinfo::{ProcessesToUpdate, System};
use tokio::time::sleep;

static SYSTEM: Lazy<Mutex<System>> = Lazy::new(|| Mutex::new(System::new()));

#[derive(Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
}

pub enum ProcessEvent {
    Started(ProcessInfo),
    Exited(ProcessInfo),
}

callback!(ProcessCallback<T>);

pub struct ProcessTrigger {
    callback: ProcessCallback<ProcessEvent>,
    known_processes: HashMap<u32, String>,
    poll_interval: Duration,
}

impl ProcessTrigger {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(ProcessEvent) -> Result<()> + Send + Sync + 'static,
    {
        Self::with_interval(f, Duration::from_millis(500))
    }

    pub fn with_interval<F>(f: F, poll_interval: Duration) -> Self
    where
        F: Fn(ProcessEvent) -> Result<()> + Send + Sync + 'static,
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

        sys.processes()
            .iter()
            .map(|(pid, proc)| {
                (pid.as_u32(), proc.name().to_string_lossy().to_string())
            })
            .collect()
    }
}

#[async_trait]
impl Trigger for ProcessTrigger {
    async fn start(&mut self) -> Result<()> {
        self.known_processes = Self::refresh_and_get_processes();

        loop {
            let current_processes = Self::refresh_and_get_processes();

            for (pid, name) in &current_processes {
                if !self.known_processes.contains_key(pid) {
                    (self.callback)(ProcessEvent::Started(ProcessInfo {
                        pid: *pid,
                        name: name.clone(),
                    }))?;
                }
            }

            for (pid, name) in &self.known_processes {
                if !current_processes.contains_key(pid) {
                    (self.callback)(ProcessEvent::Exited(ProcessInfo {
                        pid: *pid,
                        name: name.clone(),
                    }))?;
                }
            }

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