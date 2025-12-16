//! Clipboard management using 1Password's arboard library.

use crate::{Error, Result};
use arboard::Clipboard;
use once_cell::sync::Lazy;
use parking_lot::Mutex;

/// Global clipboard instance, lazily initialized on first access.
static CLIPBOARD: Lazy<Mutex<Clipboard>> = Lazy::new(|| Mutex::new(Clipboard::new().unwrap()));

/// Gets the current text content from the system clipboard.
///
/// Returns an error if the clipboard is empty, contains non-text data, or can't be accessed
/// due to system restrictions.
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
/// Returns an error if the clipboard can't be accessed or modified due to system restrictions
/// or permission issues.
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
