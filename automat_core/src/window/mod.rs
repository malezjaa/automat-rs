mod exe_path;
mod list;
mod size;
mod state;
mod titlebar;
mod window_id;

pub use exe_path::*;
pub use list::*;
pub use size::*;
pub use state::*;
pub use titlebar::*;
pub use window_id::*;

/// Unified window interface providing a high-level API for window operations.
///
/// This struct wraps a `WindowIdentifier` and provides convenient methods
/// for querying and manipulating windows across different platforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window {
  id: WindowIdentifier,
}

impl Window {
  /// Creates a new Window from a WindowIdentifier.
  pub fn new(id: WindowIdentifier) -> Self {
    Self { id }
  }

  /// Gets the underlying WindowIdentifier.
  pub fn id(&self) -> WindowIdentifier {
    self.id
  }

  /// Gets the currently focused window.
  pub fn current() -> Option<Self> {
    get_current_window_identifier().map(|id| Self::new(id))
  }

  /// Gets the title of this window.
  pub fn title(&self) -> Option<String> {
    get_window_title(self.id)
  }

  /// Gets the size of this window dimension in pixels
  pub fn size(&self) -> Option<(u32, u32)> {
    get_window_size(self.id)
  }

  /// Gets the state of this window.
  pub fn state(&self) -> crate::Result<WindowState> {
    get_window_state(self.id)
  }

  /// Checks if this window is visible.
  ///
  /// This is a convenience method that doesn't require retrieving the full state.
  #[cfg(target_os = "windows")]
  pub fn is_visible(&self) -> bool {
    is_window_visible(self.id)
  }

  /// Checks if this window is visible.
  ///
  /// On non-Windows platforms, this retrieves the full state.
  #[cfg(not(target_os = "windows"))]
  pub fn is_visible(&self) -> bool {
    self.state().map(|s| s.visible).unwrap_or(false)
  }

  /// Checks if this window is currently focused.
  pub fn is_focused(&self) -> bool {
    get_current_window_identifier()
      .map(|current| current == self.id)
      .unwrap_or(false)
  }

  /// Gets the binary executable path of this window's process.
  pub fn executable_path(&self) -> Option<String> {
    get_window_exe_path(self.id)
  }
}

/// Convenience functions for window operations
impl Window {
  /// Gets the title of the currently focused window.
  pub fn current_title() -> Option<String> {
    Self::current().and_then(|w| w.title())
  }

  /// Gets the size of the currently focused window.
  pub fn current_size() -> Option<(u32, u32)> {
    Self::current().and_then(|w| w.size())
  }

  /// Gets the state of the currently focused window.
  pub fn current_state() -> crate::Result<WindowState> {
    Self::current()
      .ok_or(crate::Error::WindowStateError(
        "No focused window".to_string(),
      ))
      .and_then(|w| w.state())
  }

  /// Lists all open windows on the system.
  pub fn list_all() -> crate::Result<Vec<Window>> {
    list_windows()
  }

  /// Finds windows whose titles contain the specified text (case-insensitive).
  pub fn find_by_title(search: &str) -> crate::Result<Vec<Self>> {
    let windows = list_windows()?;
    let search_lower = search.to_lowercase();

    Ok(
      windows
        .into_iter()
        .filter(|w| {
          if let Some(title) = w.title() {
            title.to_lowercase().contains(&search_lower)
          } else {
            false
          }
        })
        .map(|w| Self::new(w.id))
        .collect(),
    )
  }

  /// Finds a window with an exact title match.
  /// Returns the first matching window, or `None` if no window with that title exists.
  pub fn find_by_exact_title(title: &str) -> crate::Result<Option<Self>> {
    let windows = list_windows()?;

    Ok(
      windows
        .into_iter()
        .find(|w| {
          if w.title().is_some() {
            w.title().unwrap() == title
          } else {
            false
          }
        })
        .map(|w| Self::new(w.id)),
    )
  }
}
