use crate::Result;
use enigo::{Enigo, Mouse, Settings};
use once_cell::sync::Lazy;
use parking_lot::Mutex;

mod keyboard;
mod mouse;

pub use keyboard::*;
pub use mouse::*;

static ENIGO: Lazy<Mutex<Enigo>> =
    Lazy::new(|| Mutex::new(Enigo::new(&Settings::default()).unwrap()));

#[inline]
pub(crate) fn with_enigo<F, R>(f: F) -> R
where
    F: FnOnce(&mut Enigo) -> R,
{
    let mut enigo = ENIGO.lock();
    f(&mut *enigo)
}

/// Returns the current cursor position on the screen.
///
/// ```
/// let (x, y) = get_cursor_location()?;
/// println!("Cursor is at: ({}, {})", x, y);
/// ```
///
/// # Returns
///
/// A tuple `(x, y)` representing the cursor's coordinates in pixels,
/// where (0, 0) is typically the top-left corner of the primary display.
pub fn get_cursor_location() -> Result<(i32, i32)> {
    with_enigo(|enigo| enigo.location()).map_err(Into::into)
}

/// Returns the resolution of the main display.
///
/// ```
/// let (width, height) = get_display_res()?;
/// println!("Display resolution: {}x{}", width, height);
/// ```
///
/// # Returns
///
/// A tuple `(width, height)` representing the display dimensions in pixels.
pub fn get_display_res() -> Result<(i32, i32)> {
    with_enigo(|enigo| enigo.main_display()).map_err(Into::into)
}
