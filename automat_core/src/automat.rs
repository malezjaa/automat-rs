use crate::{
    await_shutdown, FileSystemTrigger, IntervalTrigger, ProcessEvent, ProcessTrigger, Result,
    Trigger, Window, WindowTrigger,
};
use notify::{Config, Event};
use std::path::PathBuf;
use std::time::Duration;

/// Builder for creating automation workflows.
pub struct Automat {
    triggers: Vec<Box<dyn Trigger>>,
}

impl Automat {
    /// Creates a new `Automat` instance.
    pub fn new() -> Self {
        Self {
            triggers: Vec::new(),
        }
    }

    /// Monitor process starts and exits.
    pub fn on_process<F>(mut self, f: F) -> Self
    where
        F: Fn(ProcessEvent) -> Result<()> + Send + Sync + 'static,
    {
        let trigger = ProcessTrigger::new(f);
        self.triggers.push(Box::new(trigger));
        self
    }

    /// Monitor process starts and exits with a custom polling interval.
    pub fn on_process_with_interval<F>(mut self, f: F, interval: Duration) -> Self
    where
        F: Fn(ProcessEvent) -> Result<()> + Send + Sync + 'static,
    {
        let trigger = ProcessTrigger::with_interval(f, interval);
        self.triggers.push(Box::new(trigger));
        self
    }

    /// Run a callback at regular intervals.
    pub fn on_interval<F>(mut self, interval: Duration, f: F) -> Self
    where
        F: Fn(Duration) -> Result<()> + Send + Sync + 'static,
    {
        let trigger = IntervalTrigger::new(interval, f);
        self.triggers.push(Box::new(trigger));
        self
    }

    /// Detect when the focused window changes.
    pub fn on_window_focus<F>(mut self, f: F) -> Self
    where
        F: Fn(Window) -> Result<()> + Send + Sync + 'static,
    {
        let trigger = WindowTrigger::new(f);
        self.triggers.push(Box::new(trigger));
        self
    }

    /// Watch for file system changes. Chain with `.watch_path()` and `.done()`.
    pub fn on_file_system<F>(self, f: F) -> FileSystemTriggerBuilder
    where
        F: Fn(Result<Event>) -> Result<()> + Send + Sync + 'static,
    {
        let trigger = FileSystemTrigger::new(f);
        FileSystemTriggerBuilder {
            automat: self,
            fs_trigger: trigger,
        }
    }

    /// Start all triggers and run until shutdown (Ctrl+C).
    pub async fn run(self) -> Result<()> {
        for mut trigger in self.triggers {
            tokio::spawn(async move {
                if let Err(e) = trigger.start().await {
                    eprintln!("Trigger '{}' failed: {}", trigger.name(), e);
                }
            });
        }

        await_shutdown().await
    }

    /// Add a custom trigger implementation.
    pub fn with_trigger<T: Trigger + 'static>(mut self, trigger: T) -> Self {
        self.triggers.push(Box::new(trigger));
        self
    }
}

impl Default for Automat {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for configuring file system triggers.
pub struct FileSystemTriggerBuilder {
    automat: Automat,
    fs_trigger: FileSystemTrigger,
}

impl FileSystemTriggerBuilder {
    /// Add a path to watch. Set `recursive` to `true` to watch subdirectories.
    pub fn watch_path(mut self, path: PathBuf, recursive: bool) -> Self {
        self.fs_trigger = self.fs_trigger.watch_path(path, recursive);
        self
    }

    /// Configure the watcher with custom settings.
    pub fn with_config(mut self, config: Config) -> Self {
        self.fs_trigger = self.fs_trigger.with_config(config);
        self
    }

    /// Finish configuring the file system trigger.
    pub fn done(mut self) -> Automat {
        self.automat
            .triggers
            .push(Box::new(self.fs_trigger));
        self.automat
    }

    /// Finish configuration and start running. Equivalent to `.done().run().await`.
    pub async fn run(self) -> Result<()> {
        self.done().run().await
    }
}
