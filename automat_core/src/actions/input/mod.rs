use enigo::{Enigo, Settings};
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
