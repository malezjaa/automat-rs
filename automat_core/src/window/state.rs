use crate::{Error, Result, WindowIdentifier};

#[cfg(target_os = "windows")]
/// Checks if a window is visible on Windows.
///
/// This is a lightweight helper that only checks visibility without retrieving the full window state.
///
/// # Safety
///
/// Uses unsafe Windows API calls with raw HWND handles.
pub fn is_window_visible(window_id: WindowIdentifier) -> bool {
  use windows::Win32::Foundation::HWND;
  use windows::Win32::UI::WindowsAndMessaging::IsWindowVisible;

  unsafe {
    let hwnd = HWND(window_id.as_u64() as *mut _);
    IsWindowVisible(hwnd).as_bool()
  }
}

/// Represents the state of a window across all platforms.
///
/// This struct provides a unified interface for window states on Windows, macOS, and Linux.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowState {
  pub visible: bool,
  pub maximized: bool,
  pub minimized: bool,
  /// Whether the window is enabled (can receive input)
  pub enabled: bool,
}

impl WindowState {
  /// Returns true if the window is in a normal state (visible, not maximized, not minimized).
  pub fn is_normal(&self) -> bool {
    self.visible && !self.maximized && !self.minimized
  }

  /// Returns true if the window is hidden (not visible).
  pub fn is_hidden(&self) -> bool {
    !self.visible
  }

  /// Returns true if the window can accept user input (enabled and visible).
  pub fn can_accept_input(&self) -> bool {
    self.enabled && self.visible
  }
}

#[cfg(target_os = "windows")]
/// Gets the state of a window on Windows.
///
/// Uses the Windows API to get window visibility, enabled state, and placement information
/// via `IsWindowVisible`, `IsWindowEnabled`, and `GetWindowPlacement`.
///
/// Returns the window state, or an error if the window handle is invalid or the API call fails.
///
/// # Safety
///
/// Uses unsafe Windows API calls with raw HWND handles.
pub fn get_window_state(window_id: WindowIdentifier) -> Result<WindowState> {
  use windows::Win32::Foundation::HWND;
  use windows::Win32::UI::WindowsAndMessaging::{
    GWL_STYLE, GetWindowLongPtrW, GetWindowPlacement, IsWindowVisible, SW_SHOWMAXIMIZED,
    SW_SHOWMINIMIZED, WINDOWPLACEMENT, WS_DISABLED,
  };

  unsafe {
    let hwnd = HWND(window_id.as_u64() as *mut _);

    let visible = IsWindowVisible(hwnd).as_bool();

    // IsWindowEnabled is not directly available in windows-rs
    // We need to use a workaround or check the window style
    let enabled = {
      let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
      (style & WS_DISABLED.0 as isize) == 0
    };

    let mut placement = WINDOWPLACEMENT::default();
    placement.length = size_of::<WINDOWPLACEMENT>() as u32;

    let result = GetWindowPlacement(hwnd, &mut placement);

    if result.is_err() {
      return Err(Error::WindowStateError(
        "Failed to get window placement".to_string(),
      ));
    }

    let maximized = placement.showCmd == SW_SHOWMAXIMIZED.0 as u32;
    let minimized = placement.showCmd == SW_SHOWMINIMIZED.0 as u32;

    Ok(WindowState {
      visible,
      maximized,
      minimized,
      enabled,
    })
  }
}

#[cfg(target_os = "linux")]
/// Gets the state of a window on Linux (X11).
///
/// Uses the X11 library to get window visibility, maximization, minimization, and input capability states.
/// Queries the window attributes and the `_NET_WM_STATE` property to determine the window's state.
///
/// Returns the window state, or an error if the display can't be opened or window attributes can't be retrieved.
///
/// # Safety
///
/// Uses unsafe X11 API calls with raw pointers and window handles.
///
/// # Notes
///
/// * `visible` maps to the X11 `IsViewable` map state
/// * `maximized` is determined by checking for `_NET_WM_STATE_MAXIMIZED_HORZ` or `_NET_WM_STATE_MAXIMIZED_VERT`
/// * `minimized` corresponds to the `_NET_WM_STATE_HIDDEN` state
/// * `enabled` checks the `InputHint` in `XWMHints`, defaulting to `true` if no hints are available
pub fn get_window_state(window_id: WindowIdentifier) -> Result<WindowState> {
  use std::ffi::CString;
  use std::ptr;
  use x11::xlib::*;

  unsafe {
    let display = XOpenDisplay(ptr::null());
    if display.is_null() {
      return Err(Error::WindowStateError(
        "Failed to open X display".to_string(),
      ));
    }

    let window = window_id.as_u64();

    // Check if a window exists
    let mut attrs = XWindowAttributes {
      x: 0,
      y: 0,
      width: 0,
      height: 0,
      border_width: 0,
      depth: 0,
      visual: ptr::null_mut(),
      root: 0,
      class: 0,
      bit_gravity: 0,
      win_gravity: 0,
      backing_store: 0,
      backing_planes: 0,
      backing_pixel: 0,
      save_under: 0,
      colormap: 0,
      map_installed: 0,
      map_state: 0,
      all_event_masks: 0,
      your_event_mask: 0,
      do_not_propagate_mask: 0,
      override_redirect: 0,
      screen: ptr::null_mut(),
    };

    let status = XGetWindowAttributes(display, window, &mut attrs);
    if status == 0 {
      XCloseDisplay(display);
      return Err(WindowStateError(
        "Failed to get window attributes".to_string(),
      ));
    }

    let visible = attrs.map_state == IsViewable;

    // Check for _NET_WM_STATE to determine maximized/minimized
    let net_wm_state = CString::new("_NET_WM_STATE").unwrap();
    let atom_net_wm_state = XInternAtom(display, net_wm_state.as_ptr(), 0);

    let mut actual_type: u64 = 0;
    let mut actual_format: i32 = 0;
    let mut nitems: u64 = 0;
    let mut bytes_after: u64 = 0;
    let mut prop: *mut u8 = ptr::null_mut();

    XGetWindowProperty(
      display,
      window,
      atom_net_wm_state,
      0,
      1024,
      0,
      AnyPropertyType as u64,
      &mut actual_type,
      &mut actual_format,
      &mut nitems,
      &mut bytes_after,
      &mut prop,
    );

    let mut maximized = false;
    let mut minimized = false;

    if !prop.is_null() {
      let states = std::slice::from_raw_parts(prop as *const u64, nitems as usize);

      let maximized_horz = XInternAtom(
        display,
        CString::new("_NET_WM_STATE_MAXIMIZED_HORZ")
          .unwrap()
          .as_ptr(),
        0,
      );
      let maximized_vert = XInternAtom(
        display,
        CString::new("_NET_WM_STATE_MAXIMIZED_VERT")
          .unwrap()
          .as_ptr(),
        0,
      );
      let hidden = XInternAtom(
        display,
        CString::new("_NET_WM_STATE_HIDDEN").unwrap().as_ptr(),
        0,
      );

      for &state in states {
        if state == maximized_horz || state == maximized_vert {
          maximized = true;
        }
        if state == hidden {
          minimized = true;
        }
      }

      XFree(prop as *mut _);
    }

    // X11 doesn't have a direct "enabled" concept
    // We check if the window accepts input
    let wm_hints: *mut XWMHints = XGetWMHints(display, window);
    let enabled = if !wm_hints.is_null() {
      let accepts_input = (*wm_hints).flags & InputHint != 0 && (*wm_hints).input != 0;
      XFree(wm_hints as *mut _);
      accepts_input
    } else {
      true // Default to enabled if no hints
    };

    XCloseDisplay(display);

    Ok(WindowState {
      visible,
      maximized,
      minimized,
      enabled,
    })
  }
}

#[cfg(target_os = "macos")]
/// Gets the state of a window on macOS.
///
/// Uses the Cocoa/AppKit framework to get window visibility, miniaturization (minimized),
/// zoom (maximized), and input capability states.
///
/// Returns the window state, or an error if the window ID is invalid or the window isn't found.
///
/// # Safety
///
/// Uses unsafe Objective-C runtime calls with raw pointers.
///
/// # Notes
///
/// * `enabled` maps to `canBecomeKeyWindow` as macOS doesn't have an exact equivalent
/// * `maximized` corresponds to the "zoomed" state in macOS terminology
pub fn get_window_state(window_id: WindowIdentifier) -> Result<WindowState> {
  use cocoa::appkit::{NSApplication, NSWindow};
  use cocoa::base::{id, nil};
  use cocoa::foundation::NSAutoreleasePool;
  use objc::{msg_send, sel, sel_impl};
  let window_id = window_id.as_u64();

  unsafe {
    let _pool = NSAutoreleasePool::new(nil);
    let app: id = msg_send![class!(NSApplication), sharedApplication];
    let windows: id = msg_send![app, windows];
    let count: usize = msg_send![windows, count];

    if window_id as usize >= count {
      return Err(WindowStateError("Invalid window ID".to_string()));
    }

    let window: id = msg_send![windows, objectAtIndex: window_id as usize];

    if window == nil {
      return Err(WindowStateError("Window not found".to_string()));
    }

    let visible: bool = msg_send![window, isVisible];
    let miniaturized: bool = msg_send![window, isMiniaturized];
    let zoomed: bool = msg_send![window, isZoomed];

    // macOS doesn't have an exact "enabled" concept like Windows
    // We check if the window can receive input
    let can_become_key: bool = msg_send![window, canBecomeKeyWindow];

    Ok(WindowState {
      visible,
      maximized: zoomed,
      minimized: miniaturized,
      enabled: can_become_key,
    })
  }
}
