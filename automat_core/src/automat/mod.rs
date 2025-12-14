mod fs_builder;
mod runner;

pub use fs_builder::FileSystemBuilder;

use crate::{
  pair_api, Error, FileSystemTrigger, IntervalTrigger, ProcessEvent, ProcessTrigger, Result,
  Trigger, TriggerContext, Window, WindowTrigger,
};
use notify::Event;
use std::future::Future;
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
    on_process<F, Fut>(f: F)
      where {
        F: Fn(TriggerContext<ProcessEvent>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
      }
      => ProcessTrigger::new(f);
    /// Monitor process starts and exits with a synchronous (blocking) callback.
    on_process_blocking<F>(f: F)
      where {
        F: Fn(TriggerContext<ProcessEvent>) -> Result<()> + Send + Sync + 'static,
      }
      => ProcessTrigger::new_blocking(f);
  }

  pair_api! {
    method
    /// Monitor process starts and exits with a custom polling interval.
    on_process_with_interval<F, Fut>(f: F, interval: Duration)
      where {
        F: Fn(TriggerContext<ProcessEvent>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
      }
      => ProcessTrigger::with_interval(f, interval);
    /// Monitor process starts and exits with a custom polling interval (blocking callback).
    on_process_with_interval_blocking<F>(f: F, interval: Duration)
      where {
        F: Fn(TriggerContext<ProcessEvent>) -> Result<()> + Send + Sync + 'static,
      }
      => ProcessTrigger::with_interval_blocking(f, interval);
  }

  pair_api! {
    method
    /// Run a callback at regular intervals.
    on_interval<F, Fut>(interval: Duration, f: F)
      where {
        F: Fn(Duration) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
      }
      => IntervalTrigger::new(interval, f);
    /// Run a callback at regular intervals (blocking callback).
    on_interval_blocking<F>(interval: Duration, f: F)
      where {
        F: Fn(Duration) -> Result<()> + Send + Sync + 'static,
      }
      => IntervalTrigger::new_blocking(interval, f);
  }

  pair_api! {
    method
    /// Detect when the focused window changes.
    on_window_focus<F, Fut>(f: F)
      where {
        F: Fn(Window) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
      }
      => WindowTrigger::new(f);
    /// Detect when the focused window changes (blocking callback).
    on_window_focus_blocking<F>(f: F)
      where {
        F: Fn(Window) -> Result<()> + Send + Sync + 'static,
      }
      => WindowTrigger::new_blocking(f);
  }

  pair_api! {
    method
    /// Monitor filesystem changes.
    on_fs_watch<F, Fut>(f: F)
      where {
        F: Fn(Result<Event>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
      }
      => FileSystemTrigger::new(f);
    /// Monitor filesystem changes (blocking callback).
    on_fs_watch_blocking<F>(f: F)
      where {
        F: Fn(Result<Event>) -> Result<()> + Send + Sync + 'static,
      }
      => FileSystemTrigger::new_blocking(f);
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
