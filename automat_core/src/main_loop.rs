use super::error::{Error, Result};
use tokio::signal::ctrl_c;

/// Keeps the automation runtime alive to handle concurrent workflow triggers.
///
/// This function blocks the main task while allowing workflow triggers (webhooks,
/// schedules, file watchers, etc.) to run concurrently in the background.
/// The application will run until Ctrl-C (SIGINT) is received.
pub async fn await_shutdown() -> Result<()> {
    ctrl_c().await.map_err(Error::IoError)
}
