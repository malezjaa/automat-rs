mod fs_builder;
mod runner;

pub use fs_builder::FileSystemBuilder;

use crate::{
  pair_api, Error, FileSystemTrigger, IntervalTrigger, ProcessEvent, ProcessTrigger, Result,
  Trigger, TriggerContext, Window, WindowTrigger,
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

  pair_api! {
    method
    /// Monitor process starts and exits.
    on_process(f: F)
      callback(TriggerContext<ProcessEvent>)
      => (ProcessTrigger)::new(f);
  }

  pair_api! {
    method
    /// Monitor process starts and exits with a custom polling interval.
    on_process_with_interval(f: F, interval: Duration)
      callback(TriggerContext<ProcessEvent>)
      => (ProcessTrigger)::with_interval(f, interval);
  }

  pair_api! {
    method
    /// Run a callback at regular intervals.
    on_interval(interval: Duration, f: F)
      callback(Duration)
      => (IntervalTrigger)::new(interval, f);
  }

  pair_api! {
    method
    /// Detect when the focused window changes.
    on_window_focus(f: F)
      callback(Window)
      => (WindowTrigger)::new(f);
  }

  pair_api! {
    method
    /// Monitor filesystem changes.
    on_fs_watch(f: F)
      callback(Result<Event>)
      => (FileSystemTrigger)::new(f);
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
