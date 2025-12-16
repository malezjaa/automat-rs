use windows::Win32::Foundation::{LPARAM, WPARAM};
use crate::{Action, Result, Window, WindowIdentifier};

/// Closes a window.
pub struct CloseWindow {
  window_id: WindowIdentifier,
}

impl CloseWindow {
  /// Closes a specific window.
  pub fn for_window(window: Window) -> Self {
    Self {
      window_id: window.id(),
    }
  }

  /// Closes the currently focused window.
  pub fn current() -> Self {
    Self::try_current().expect("No focused window")
  }

  /// Attempts to close the currently focused window.
  pub fn try_current() -> Option<Self> {
    Window::current().map(Self::for_window)
  }

  /// Creates an action for a specific window identifier.
  pub fn from_id(window_id: WindowIdentifier) -> Self {
    Self { window_id }
  }
}

impl Action for CloseWindow {
  fn run(&self) -> Result<()> {
    close_window(self.window_id)
  }
}

#[cfg(target_os = "windows")]
fn close_window(window_id: WindowIdentifier) -> Result<()> {
  use windows::Win32::Foundation::HWND;
  use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_CLOSE};

  unsafe {
    let hwnd = HWND(window_id.as_u64() as *mut _);
    PostMessageW(
      Some(hwnd),
      WM_CLOSE,
      WPARAM(0),
      LPARAM(0),
    )
    .map_err(|e| crate::Error::WindowStateError(format!("Failed to close window: {}", e)))?;
  }

  Ok(())
}

#[cfg(target_os = "linux")]
fn close_window(window_id: WindowIdentifier) -> Result<()> {
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
    let root = XDefaultRootWindow(display);

    // Get the WM_DELETE_WINDOW and WM_PROTOCOLS atoms
    let wm_protocols = CString::new("WM_PROTOCOLS").unwrap();
    let wm_delete_window = CString::new("WM_DELETE_WINDOW").unwrap();

    let atom_wm_protocols = XInternAtom(display, wm_protocols.as_ptr(), 0);
    let atom_wm_delete_window = XInternAtom(display, wm_delete_window.as_ptr(), 0);

    // Send ClientMessage to close the window
    let mut event: XClientMessageEvent = std::mem::zeroed();
    event.type_ = ClientMessage;
    event.window = window;
    event.message_type = atom_wm_protocols;
    event.format = 32;
    event.data.set_long(0, atom_wm_delete_window as i64);
    event.data.set_long(1, CurrentTime as i64);

    XSendEvent(
      display,
      window,
      0,
      NoEventMask,
      &mut event as *mut XClientMessageEvent as *mut XEvent,
    );

    XFlush(display);
    XCloseDisplay(display);
  }

  Ok(())
}

#[cfg(target_os = "macos")]
fn close_window(window_id: WindowIdentifier) -> Result<()> {
  use crate::Error;
  use objc2::rc::autoreleasepool;
  use objc2::{msg_send, sel};
  use objc2_app_kit::{NSApplication, NSWindow};
  use objc2_foundation::NSArray;

  autoreleasepool(|_| unsafe {
    let app: *mut NSApplication = msg_send![class!(NSApplication), sharedApplication];
    let windows: *mut NSArray<NSWindow> = msg_send![app, windows];
    let count: usize = msg_send![windows, count];

    for i in 0..count {
      let window: *mut NSWindow = msg_send![windows, objectAtIndex: i];
      let win_number: i64 = msg_send![window, windowNumber];

      if win_number as u64 == window_id.as_u64() {
        let _: () = msg_send![window, close];
        return Ok(());
      }
    }

    Err(Error::WindowStateError("Window not found".to_string()))
  })
}
