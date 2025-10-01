use super::error::Result;
use async_trait::async_trait;

/// Represents a trigger that initiates workflow execution.
///
/// Triggers listen for events from various sources (webhooks, schedules,
/// file changes, etc.) and start workflows when those events occur.
///
/// # Example
///
/// ```rust,ignore
/// struct WebhookTrigger {
///     port: u16,
///     server_handle: Option<ServerHandle>,
/// }
///
/// #[async_trait]
/// impl Trigger for WebhookTrigger {
///     async fn start(&mut self) -> Result<()> {
///         let listener = TcpListener::bind(("0.0.0.0", self.port)).await?;
///         // Handle incoming requests and execute workflows
///         Ok(())
///     }
///
///     async fn stop(&mut self) -> Result<()> {
///         if let Some(handle) = self.server_handle.take() {
///             handle.shutdown().await;
///         }
///         Ok(())
///     }
///
///     fn name(&self) -> &str {
///         "webhook"
///     }
/// }
/// ```
#[async_trait]
pub trait Trigger: Send + Sync {
    /// Starts the trigger and begins listening for events.
    ///
    /// This method should block until `stop()` is called or an error occurs.
    /// Implementations should handle their own concurrency (spawning tasks,
    /// setting up listeners, etc.).
    async fn start(&mut self) -> Result<()>;

    /// Runs on every trigger. Should be called from `start`
    async fn run(&mut self) -> Result<()>;

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
    fn name(&self) -> &str;

    /// Returns whether the trigger is currently running.
    ///
    /// Default implementation returns `false`. Override if you need to
    /// track running state.
    fn is_running(&self) -> bool {
        false
    }
}

/// Spawns a new thread with the trigger instance running on it.
pub async fn new_trigger(mut trigger: impl Trigger + 'static) {
    tokio::spawn( async move { trigger.start().await });
}
