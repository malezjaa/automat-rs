use crate::{
  await_shutdown, Error, IntervalTrigger, ProcessEvent, ProcessTrigger, Result, Trigger,
  TriggerContext, TriggerEvent, Window, WindowTrigger,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{channel, Receiver};
use tokio_util::sync::CancellationToken;

/// Error handler type for trigger callbacks.
pub type ErrorHandler = Arc<dyn Fn(Error) + Send + Sync>;

/// Builder for creating automation workflows.
pub struct Automat {
  triggers: Vec<Box<dyn Trigger>>,
  error_handler: Option<ErrorHandler>,
}

impl Automat {
  /// Creates a new `Automat` instance.
  pub fn new() -> Self {
    Self {
      triggers: Vec::new(),
      error_handler: None,
    }
  }

  /// Set a custom error handler for all trigger callbacks.
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

  pub async fn run(self) -> Result<()> {
    let error_handler = self.error_handler.clone();
    let mut handles = Vec::new();

    for mut trigger in self.triggers {
      let (tx, rx) = channel(100);
      let handler = error_handler.clone();
      let cancel_token = CancellationToken::new();
      let cancel_token_clone = cancel_token.clone();

      let trigger_handle = tokio::spawn(async move {
        tokio::select! {
            _ = trigger.start(tx) => {},
            _ = cancel_token_clone.cancelled() => {},
        }
      });

      let event_handle =
        tokio::spawn(async move { Self::handle_trigger_events(rx, handler, cancel_token).await });

      handles.push((trigger_handle, event_handle));
    }

    let shutdown_result = await_shutdown().await;

    for (trigger_handle, event_handle) in handles {
      trigger_handle.abort();
      event_handle.abort();
    }

    shutdown_result
  }

  async fn handle_trigger_events(
    mut rx: Receiver<TriggerEvent>,
    error_handler: Option<ErrorHandler>,
    cancel_token: CancellationToken,
  ) {
    while let Some(event) = rx.recv().await {
      match event {
        TriggerEvent::Error(err) => {
          if let Some(ref handler) = error_handler {
            handler(err);
          } else {
            eprintln!("Trigger error: {}", err);
          }
        }
        TriggerEvent::ErrorFatal(err) => {
          if let Some(ref handler) = error_handler {
            handler(err);
          } else {
            eprintln!("Fatal trigger error: {}", err);
          }
          cancel_token.cancel();
          break;
        }
        TriggerEvent::Stop => {
          cancel_token.cancel();
          break;
        }
      }
    }
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
