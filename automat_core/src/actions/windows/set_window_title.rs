use crate::{Action, Result, Window, WindowIdentifier};

/// Sets the title of a window.
///
/// Works on Windows (via `SetWindowTextW`), Linux (via X11 `XStoreName`),
/// and macOS (limited support, may need accessibility permissions).
///
/// ```no_run
/// use automat_core::*;
///
/// // Set title of current window
/// SetWindowTitle::current("New Title").run().unwrap();
///
/// // Set title of specific window
/// let window = Window::current().unwrap();
/// SetWindowTitle::for_window(window, "Custom Title").run().unwrap();
/// ```
pub struct SetWindowTitle {
  window_id: WindowIdentifier,
  title: String,
}

impl SetWindowTitle {
  /// Sets the title of a specific window.
  pub fn for_window(window: Window, title: impl Into<String>) -> Self {
    Self {
      window_id: window.id(),
      title: title.into(),
    }
  }

  /// Sets the title of the currently focused window.
  ///
  /// Panics if no window is focused. Use `try_current` for a fallible version.
  pub fn current(title: impl Into<String>) -> Self {
    Self::try_current(title).expect("No focused window")
  }

  /// Attempts to set the title of the currently focused window.
  ///
  /// Returns `None` if no window is focused.
  pub fn try_current(title: impl Into<String>) -> Option<Self> {
    Window::current().map(|window| Self::for_window(window, title))
  }

  /// Creates an action for a specific window identifier.
  pub fn from_id(window_id: WindowIdentifier, title: impl Into<String>) -> Self {
    Self {
      window_id,
      title: title.into(),
    }
  }

  /// Returns the target window identifier.
  pub fn window_id(&self) -> WindowIdentifier {
    self.window_id
  }

  /// Returns the title that will be set.
  pub fn title(&self) -> &str {
    &self.title
  }
}

impl Action for SetWindowTitle {
  fn run(&self) -> Result<()> {
    set_window_title_impl(self.window_id, &self.title)
  }
}

#[cfg(target_os = "windows")]
fn set_window_title_impl(window_id: WindowIdentifier, title: &str) -> Result<()> {
  use windows::Win32::Foundation::HWND;
  use windows::Win32::UI::WindowsAndMessaging::SetWindowTextW;
  use windows::core::HSTRING;

  unsafe {
    let hwnd = HWND(window_id.as_u64() as *mut _);
    let title_hstring = HSTRING::from(title);
    let result = SetWindowTextW(hwnd, &title_hstring);

    if result.is_ok() {
      Ok(())
    } else {
      Err(crate::Error::WindowTitleError(
        "Failed to set window title".to_string(),
      ))
    }
  }
}

#[cfg(target_os = "linux")]
fn set_window_title_impl(window_id: WindowIdentifier, title: &str) -> Result<()> {
  use std::ffi::CString;
  use std::ptr;
  use x11::xlib::*;

  unsafe {
    let display = XOpenDisplay(ptr::null());
    if display.is_null() {
      return Err(crate::Error::WindowTitleError(
        "Failed to open X display".to_string(),
      ));
    }

    let window = window_id.as_u64();
    let title_cstr = CString::new(title)
      .map_err(|_| crate::Error::WindowTitleError("Invalid title string".to_string()))?;

    XStoreName(display, window, title_cstr.as_ptr());
    XFlush(display);
    XCloseDisplay(display);

    Ok(())
  }
}

#[cfg(target_os = "macos")]
fn set_window_title_impl(_window_id: WindowIdentifier, title: &str) -> Result<()> {
  use cocoa::appkit::NSWindow;
  use cocoa::base::{id, nil};
  use cocoa::foundation::NSString;
  use objc::{class, msg_send, sel, sel_impl};

  unsafe {
    let app: id = msg_send![class!(NSApplication), sharedApplication];
    let main_window: id = msg_send![app, mainWindow];

    if main_window == nil {
      return Err(crate::Error::WindowTitleError(
        "No main window found".to_string(),
      ));
    }

    let ns_title = NSString::alloc(nil);
    let ns_title = NSString::init_str(ns_title, title);
    let _: () = msg_send![main_window, setTitle: ns_title];

    Ok(())
  }
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn set_window_title_impl(_window_id: WindowIdentifier, _title: &str) -> Result<()> {
  Err(crate::Error::WindowTitleError(
    "Setting window title is not supported on this platform".to_string(),
  ))
}
