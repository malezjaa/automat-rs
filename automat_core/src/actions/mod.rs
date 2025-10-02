mod input;
mod open_in_browser;

use super::error::Result;
use async_trait::async_trait;

pub use enigo::{Axis, Button, Coordinate, Direction, Key};
pub use input::*;
pub use open_in_browser::*;

/// Performs an action when called by the trigger.
pub trait Action: Send + Sync {
    fn run(&self) -> Result<()>;
}

/// Same as Action but runs in async.
#[async_trait]
pub trait ActionAsync: Send + Sync {
    async fn run_async(&self) -> Result<()>;
}
