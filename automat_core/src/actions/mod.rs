mod input;
mod open_in_browser;
mod set_window_title;

use super::error::Result;
use async_trait::async_trait;

pub use enigo::{Axis, Button, Coordinate, Direction, Key};
pub use input::*;
pub use open_in_browser::*;
pub use set_window_title::*;

/// Represents a synchronous action that can be executed.
///
/// Actions are usually triggered automatically, but you can also
/// call [`run`](Action::run) directly in your own code.
pub trait Action<T = ()>: Send + Sync {
  /// Executes the action synchronously.
  ///
  /// Returns a [`Result`] indicating whether the action completed successfully.
  fn run(&self) -> Result<T>;
}

/// Represents an asynchronous action that can be executed.
///
/// Like [`Action`], but designed for async contexts.
/// Async actions are often called by triggers, but you can also
/// invoke [`run_async`](ActionAsync::run_async) directly wherever you need it.
#[async_trait]
pub trait ActionAsync<T = ()>: Send + Sync {
  /// Executes the action asynchronously.
  ///
  /// Can be awaited from anywhere in your code.
  /// Returns a [`Result`] indicating whether the action completed successfully.
  async fn run_async(&self) -> Result<T>;
}
