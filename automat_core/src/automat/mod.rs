mod fs_builder;
mod runner;

pub use fs_builder::FileSystemBuilder;

use crate::{
  Error, FileSystemTrigger, IntervalTrigger, ProcessEvent, ProcessTrigger, Result, Trigger,
  TriggerContext, Window, WindowTrigger,
};
use notify::Event;
use std::sync::Arc;
use std::time::Duration;

/// Error handler type for trigger callbacks.
pub type ErrorHandler = Arc<dyn Fn(Error) + Send + Sync>;

pub struct Automat {
  triggers: Vec<Box<dyn Trigger>>,
  error_handler: Option<ErrorHandler>,
}

impl Automat {
  pub fn new() -> Self {
    Self {
      triggers: Vec::new(),
      error_handler: None,
    }
  }

  pub fn on_error<F>(mut self, handler: F) -> Self
  where
    F: Fn(Error) + Send + Sync + 'static,
  {
    self.error_handler = Some(Arc::new(handler));
    self
  }

  /// Monitor process starts and exits.
  pub fn on_process<F>(mut self, f: F) -> Self
  where
    F: Fn(TriggerContext<ProcessEvent>) -> Result<()> + Send + Sync + 'static,
  {
    let trigger = ProcessTrigger::new(f);
    self.triggers.push(Box::new(trigger));
    self
  }

  /// Monitor process starts and exits with a custom polling interval.
  pub fn on_process_with_interval<F>(mut self, f: F, interval: Duration) -> Self
  where
    F: Fn(TriggerContext<ProcessEvent>) -> Result<()> + Send + Sync + 'static,
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

  pub fn on_fs_watch<F>(mut self, f: F) -> Self
  where
    F: Fn(Result<Event>) -> Result<()> + Send + Sync + 'static,
  {
    let trigger = FileSystemTrigger::new(f);
    self.triggers.push(Box::new(trigger));
    self
  }

  /// Configure a file system watcher using a builder pattern.
  pub fn with_fs_watch<B>(mut self, builder_fn: B) -> Self
  where
    B: FnOnce(FileSystemBuilder) -> FileSystemTrigger,
  {
    let builder = FileSystemBuilder::new();
    let trigger = builder_fn(builder);
    self.triggers.push(Box::new(trigger));
    self
  }

  pub fn extend(mut self, other: Automat) -> Self {
    self.triggers.extend(other.triggers);
    self
  }

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
