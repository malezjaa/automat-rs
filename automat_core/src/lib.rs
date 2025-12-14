//! # automat-core
//!
//! Core functionality for the Automat ecosystem.
//!
//! This crate defines the foundation for building automation workflows,
//! including:
//!
//! - **Triggers**: events or conditions that initiate actions, such as time-based schedules,
//!   state changes, user input, external signals, or internal events.
//! - **Actions**: operations executed in response to triggers.
//!
//! By providing these building blocks, `automat-core` enables consistent, extensible,
//! and reliable automation across the entire ecosystem.

mod actions;
mod automat;
mod callback;
mod clipboard;
mod display_macro;
mod error;
mod main_loop;
mod triggers;
mod window;

pub use actions::*;
pub use automat::*;
pub use clipboard::*;
pub use error::*;
pub use main_loop::*;
pub use triggers::*;
pub use window::*;

pub use async_trait::async_trait;
pub use display_info::{error::*, DisplayInfo};
