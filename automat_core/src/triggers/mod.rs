mod context;
mod fs_watcher;
mod interval;
mod process;
mod window;

use super::error::{Error, Result};
use async_trait::async_trait;
pub use context::*;
pub use fs_watcher::*;
pub use interval::*;
pub use process::*;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;
pub use window::*;

/// Error handler for trigger callbacks.
pub type TriggerErrorHandler = Arc<dyn Fn(Error) + Send + Sync>;

/// Runtime context passed to triggers by the runner.
///
/// Triggers should stop promptly when `shutdown` is cancelled.
#[derive(Clone)]
pub struct TriggerRuntime {
  pub tx: Sender<TriggerEvent>,
  pub shutdown: CancellationToken,
}

/// Represents a trigger that initiates workflow execution.
///
/// Triggers listen for events from various sources (webhooks, schedules,
/// file changes, etc.) and start workflows when those events occur.
///
/// Trigger requires `Action` trait implemented.
///
/// # Example
///
/// ```rust no_run
/// use automat_core::{async_trait, Result, Trigger, TriggerEvent, TriggerRuntime};
///
/// struct MyTrigger;
///
/// #[async_trait]
/// impl Trigger for MyTrigger {
///   async fn start(&mut self, rt: TriggerRuntime) -> Result<()> {
///     // Example: run until shutdown is requested.
///     rt.shutdown.cancelled().await;
///     let _ = rt.tx.send(TriggerEvent::Stop).await;
///     Ok(())
///   }
///
///   fn name(&self) -> String {
///     "my-trigger".to_string()
///   }
/// }
/// ```
#[async_trait]
pub trait Trigger: Send + Sync {
  /// Starts the trigger and begins listening for events.
  ///
  /// This method should block until `stop()` is called or an error occurs.
  /// Implementations should handle their own concurrency (spawning tasks,
  /// setting up listeners, etc.).
  async fn start(&mut self, rt: TriggerRuntime) -> Result<()>;

  /// Stops the trigger and cleans up resources.
  ///
  /// This method should gracefully shut down the trigger, cancelling any
  /// pending operations and releasing resources. It should be idempotent
  /// (safe to call multiple times).
  ///
  async fn stop(&mut self) -> Result<()> {
    Ok(())
  }

  /// Returns a unique identifier for this trigger.
  fn name(&self) -> String;

  /// Returns whether the trigger is currently running.
  ///
  /// The default implementation returns `false`. Override if you need to
  /// track the running state.
  fn is_running(&self) -> bool {
    false
  }
}
