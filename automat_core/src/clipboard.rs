//! Clipboard management using 1Password's arboard library.
//!
//! This module provides a thread-safe global clipboard interface through a lazily initialized
//! singleton.
//!
//! # Thread Safety
//!
//! All functions in this module are thread-safe and can be called from any thread without
//! external synchronization.

use crate::{Error, Result};
use arboard::Clipboard;
use once_cell::sync::Lazy;
use parking_lot::Mutex;

/// Global clipboard instance, lazily initialized on first access.
///
/// Uses `parking_lot::Mutex` for efficient locking and panics if clipboard
/// initialization fails during first access.
static CLIPBOARD: Lazy<Mutex<Clipboard>> = Lazy::new(|| Mutex::new(Clipboard::new().unwrap()));

/// Retrieves the current text content from the system clipboard.
///
/// # Errors
///
/// Returns an error if:
/// - The clipboard is empty
/// - The clipboard contains non-text data
/// - The clipboard cannot be accessed due to system restrictions
///
/// # Examples
///
/// ```no_run
/// let text = get_clipboard_text()?;
/// println!("Clipboard contains: {}", text);
/// ```
#[inline(always)]
pub fn get_clipboard_text() -> Result<String> {
    CLIPBOARD.lock().get_text().map_err(Error::ClipboardError)
}

/// Sets the system clipboard to the specified text content.
///
/// This replaces any existing clipboard content with the provided text.
///
/// # Errors
///
/// Returns an error if the clipboard cannot be accessed or modified due to
/// system restrictions or permission issues.
///
/// # Examples
///
/// ```no_run
/// set_clipboard_text("Hello, clipboard!")?;
/// ```
#[inline(always)]
pub fn set_clipboard_text(text: &str) -> Result<()> {
    CLIPBOARD
        .lock()
        .set_text(text)
        .map_err(Error::ClipboardError)
}
