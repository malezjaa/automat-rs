use crate::{Error, Result, Window, WindowIdentifier};
use windows::core::BOOL;

#[cfg(target_os = "windows")]
/// Lists all open windows on Windows.
///
/// Enumerates all top-level windows using the Windows API and collects their window handles.
///
/// # Safety
///
/// Uses unsafe Windows API calls for window enumeration.
pub fn list_windows() -> Result<Vec<Window>> {
  use crate::window::is_window_visible;
  use windows::Win32::Foundation::{HWND, LPARAM};
  use windows::Win32::UI::WindowsAndMessaging::EnumWindows;

  let mut windows = Vec::new();

  unsafe extern "system" fn enum_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
      let windows = &mut *(lparam.0 as *mut Vec<Window>);
      let window_id = WindowIdentifier::new(hwnd.0 as u64);

      if is_window_visible(window_id) {
        windows.push(Window { id: window_id });
      }

      true.into()
    }
  }

  unsafe {
    let lparam = LPARAM(&mut windows as *mut _ as isize);
    EnumWindows(Some(enum_window_proc), lparam)
      .map_err(|e| Error::WindowListError(format!("Failed to enumerate windows: {}", e)))?;
  }

  Ok(windows)
}

#[cfg(target_os = "linux")]
/// Lists all open windows on Linux using X11.
///
/// Connects to the X11 display and queries all client windows to retrieve their window IDs and titles.
///
/// # Safety
///
/// Uses unsafe X11 API calls for display connection and window queries.
pub fn list_windows() -> Result<Vec<Window>> {
  use crate::window::get_window_title;
  use std::ptr;
  use x11::xlib::{Display, Window, XCloseDisplay, XFree, XOpenDisplay, XQueryTree, XRootWindow};

  unsafe {
    let display = XOpenDisplay(ptr::null());
    if display.is_null() {
      return Err(Error::WindowListError(
        "Failed to open X11 display".to_string(),
      ));
    }

    let root = XRootWindow(display, 0);
    let mut windows = Vec::new();

    let mut root_return: Window = 0;
    let mut parent_return: Window = 0;
    let mut children: *mut Window = ptr::null_mut();
    let mut nchildren: u32 = 0;

    if XQueryTree(
      display,
      root,
      &mut root_return,
      &mut parent_return,
      &mut children,
      &mut nchildren,
    ) != 0
    {
      for i in 0..nchildren {
        let window = *children.offset(i as isize);
        let window_id = WindowIdentifier::new(window as u64);

        windows.push(Window { id: window_id });
      }

      if !children.is_null() {
        XFree(children as *mut _);
      }
    }

    XCloseDisplay(display);
    Ok(windows)
  }
}

#[cfg(target_os = "macos")]
/// Lists all open windows on macOS using Cocoa APIs.
///
/// Uses the CGWindowListCopyWindowInfo API to retrieve information about all windows on the system
/// and filters for regular application windows.
///
/// # Safety
///
/// Uses unsafe Objective-C and Core Foundation API calls.
pub fn list_windows() -> Result<Vec<Window>> {
  use core_foundation::array::{CFArray, CFArrayRef};
  use core_foundation::base::{CFType, TCFType};
  use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
  use core_foundation::number::CFNumber;
  use core_foundation::string::{CFString, CFStringRef};
  use core_graphics::window::{CGWindowListCopyWindowInfo, kCGWindowListOptionOnScreenOnly};

  unsafe {
    let window_list_info = CGWindowListCopyWindowInfo(kCGWindowListOptionOnScreenOnly, 0);
    if window_list_info.is_null() {
      return Err(Error::WindowListError(
        "Failed to get window list".to_string(),
      ));
    }

    let array: CFArray<CFDictionary> =
      CFArray::wrap_under_create_rule(window_list_info as CFArrayRef);
    let mut windows = Vec::new();

    for i in 0..array.len() {
      if let Some(dict) = array.get(i) {
        let window_layer_key = CFString::from_static_string("kCGWindowLayer");
        let window_name_key = CFString::from_static_string("kCGWindowName");
        let window_number_key = CFString::from_static_string("kCGWindowNumber");

        if let Some(layer_ref) = dict.find(&window_layer_key as *const _ as *const _) {
          let layer: CFNumber = TCFType::wrap_under_get_rule(*layer_ref as *const _);
          if let Some(layer_value) = layer.to_i32() {
            if layer_value == 0 {
              let mut window_id = 0u64;

              if let Some(number_ref) = dict.find(&window_number_key as *const _ as *const _) {
                let number: CFNumber = TCFType::wrap_under_get_rule(*number_ref as *const _);
                if let Some(num) = number.to_i64() {
                  window_id = num as u64;
                }
              }

              if window_id != 0 {
                windows.push(Window {
                  id: WindowIdentifier::new(window_id),
                });
              }
            }
          }
        }
      }
    }

    Ok(windows)
  }
}
