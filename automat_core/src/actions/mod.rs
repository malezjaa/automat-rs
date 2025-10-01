mod open_in_browser;

use super::error::Result;
use async_trait::async_trait;

pub use open_in_browser::*;

/// Performs an action when called by the trigger.
#[async_trait]
pub trait Action: Send + Sync {
    async fn run(&self) -> Result<()>;
}
