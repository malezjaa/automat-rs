use super::error::Result;
use async_trait::async_trait;

/// Performs an action when called by the trigger.
#[async_trait]
pub trait Action: Send + Sync {
    async fn run(&self) -> Result<()>;

    fn name(&self) -> String;
}
