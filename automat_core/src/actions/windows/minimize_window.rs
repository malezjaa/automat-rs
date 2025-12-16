use crate::{Action, Result, Window, WindowIdentifier};

/// Minimizes a window.
pub struct MinimizeWindow {
  window_id: WindowIdentifier,
}

impl MinimizeWindow {
  /// Minimizes a specific window.
  pub fn for_window(window: Window) -> Self {
    Self {
      window_id: window.id(),
    }
  }

  /// Minimizes the currently focused window.
  pub fn current() -> Self {
    Self::try_current().expect("No focused window")
  }

  /// Attempts to minimize the currently focused window.
  pub fn try_current() -> Option<Self> {
    Window::current().map(Self::for_window)
  }

  /// Creates an action for a specific window identifier.
  pub fn from_id(window_id: WindowIdentifier) -> Self {
    Self { window_id }
  }
}

impl Action for MinimizeWindow {
  fn run(&self) -> Result<()> {
    minimize_window(self.window_id)
  }
}

#[cfg(target_os = "windows")]
fn minimize_window(window_id: WindowIdentifier) -> Result<()> {
  use windows::Win32::Foundation::HWND;
  use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_MINIMIZE};

  unsafe {
    let hwnd = HWND(window_id.as_u64() as *mut _);
    let _ = ShowWindow(hwnd, SW_MINIMIZE);
  }

  Ok(())
}

#[cfg(target_os = "linux")]
fn minimize_window(window_id: WindowIdentifier) -> Result<()> {
  use std::ffi::CString;
  use std::ptr;
  use x11::xlib::*;

  unsafe {
    let display = XOpenDisplay(ptr::null());
    if display.is_null() {
      return Err(crate::Error::WindowStateError(
        "Failed to open X display".to_string(),
      ));
    }

    let window = window_id.as_u64();
    let screen = XDefaultScreen(display);

    // Iconify the window
    XIconifyWindow(display, window, screen);
    XFlush(display);
    XCloseDisplay(display);
  }

  Ok(())
}

#[cfg(target_os = "macos")]
fn minimize_window(window_id: WindowIdentifier) -> Result<()> {
  use crate::Error;
  use core_graphics::window::{kCGNullWindowID, CGWindowListCopyWindowInfo};
  use objc2::rc::autoreleasepool;
  use objc2::runtime::AnyObject;
  use objc2::{msg_send, msg_send_id, sel};
  use objc2_app_kit::{NSApplication, NSWindow};
  use objc2_foundation::{ns_string, NSArray, NSDictionary, NSNumber, NSString};

  autoreleasepool(|_| {
    unsafe {
      let app: *mut NSApplication = msg_send![class!(NSApplication), sharedApplication];
      let windows: *mut NSArray<NSWindow> = msg_send![app, windows];
      let count: usize = msg_send![windows, count];

      for i in 0..count {
        let window: *mut NSWindow = msg_send![windows, objectAtIndex: i];
        let win_number: i64 = msg_send![window, windowNumber];

        if win_number as u64 == window_id.as_u64() {
          let _: () = msg_send![window, miniaturize: ptr::null::<AnyObject>()];
          return Ok(());
        }
      }

      Err(Error::WindowStateError(
        "Window not found".to_string(),
      ))
    }
  })
}
