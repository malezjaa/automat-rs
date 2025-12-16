use crate::{Action, Result, Window, WindowIdentifier};

/// Maximizes a window.
pub struct MaximizeWindow {
  window_id: WindowIdentifier,
}

impl MaximizeWindow {
  /// Maximizes a specific window.
  pub fn for_window(window: Window) -> Self {
    Self {
      window_id: window.id(),
    }
  }

  /// Maximizes the currently focused window.
  pub fn current() -> Self {
    Self::try_current().expect("No focused window")
  }

  /// Attempts to maximize the currently focused window.
  pub fn try_current() -> Option<Self> {
    Window::current().map(Self::for_window)
  }

  /// Creates an action for a specific window identifier.
  pub fn from_id(window_id: WindowIdentifier) -> Self {
    Self { window_id }
  }
}

impl Action for MaximizeWindow {
  fn run(&self) -> Result<()> {
    maximize_window(self.window_id)
  }
}

#[cfg(target_os = "windows")]
fn maximize_window(window_id: WindowIdentifier) -> Result<()> {
  use windows::Win32::Foundation::HWND;
  use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_MAXIMIZE};

  unsafe {
    let hwnd = HWND(window_id.as_u64() as *mut _);
    let _ = ShowWindow(hwnd, SW_MAXIMIZE);
  }

  Ok(())
}

#[cfg(target_os = "linux")]
fn maximize_window(window_id: WindowIdentifier) -> Result<()> {
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

    // Get the _NET_WM_STATE and _NET_WM_STATE_MAXIMIZED atoms
    let net_wm_state = CString::new("_NET_WM_STATE").unwrap();
    let net_wm_state_max_horz = CString::new("_NET_WM_STATE_MAXIMIZED_HORZ").unwrap();
    let net_wm_state_max_vert = CString::new("_NET_WM_STATE_MAXIMIZED_VERT").unwrap();

    let atom_net_wm_state = XInternAtom(display, net_wm_state.as_ptr(), 0);
    let atom_max_horz = XInternAtom(display, net_wm_state_max_horz.as_ptr(), 0);
    let atom_max_vert = XInternAtom(display, net_wm_state_max_vert.as_ptr(), 0);

    // Send ClientMessage to maximize the window
    let mut event: XClientMessageEvent = std::mem::zeroed();
    event.type_ = ClientMessage;
    event.window = window;
    event.message_type = atom_net_wm_state;
    event.format = 32;
    event.data.set_long(0, 1); // _NET_WM_STATE_ADD
    event.data.set_long(1, atom_max_horz as i64);
    event.data.set_long(2, atom_max_vert as i64);

    XSendEvent(
      display,
      root,
      0,
      SubstructureNotifyMask | SubstructureRedirectMask,
      &mut event as *mut XClientMessageEvent as *mut XEvent,
    );

    XFlush(display);
    XCloseDisplay(display);
  }

  Ok(())
}

#[cfg(target_os = "macos")]
fn maximize_window(window_id: WindowIdentifier) -> Result<()> {
  use crate::Error;
  use objc2::rc::autoreleasepool;
  use objc2::runtime::AnyObject;
  use objc2::{msg_send, sel};
  use objc2_app_kit::{NSApplication, NSWindow};
  use objc2_foundation::NSArray;
  use std::ptr;

  autoreleasepool(|_| {
    unsafe {
      let app: *mut NSApplication = msg_send![class!(NSApplication), sharedApplication];
      let windows: *mut NSArray<NSWindow> = msg_send![app, windows];
      let count: usize = msg_send![windows, count];

      for i in 0..count {
        let window: *mut NSWindow = msg_send![windows, objectAtIndex: i];
        let win_number: i64 = msg_send![window, windowNumber];

        if win_number as u64 == window_id.as_u64() {
          let _: () = msg_send![window, zoom: ptr::null::<AnyObject>()];
          return Ok(());
        }
      }

      Err(Error::WindowStateError(
        "Window not found".to_string(),
      ))
    }
  })
}
