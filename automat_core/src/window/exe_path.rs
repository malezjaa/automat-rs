use crate::window::WindowIdentifier;

#[cfg(target_os = "windows")]
/// Gets the executable path of the process owning the specified window on Windows.
///
/// Uses the Windows API to get the process ID associated with the window, then retrieves
/// the full path to the executable. Returns the full executable path, or `None` if the window
/// doesn't exist, the process can't be accessed, or the path can't be retrieved.
///
/// # Safety
///
/// Uses unsafe Windows API calls. Opens process handles which are properly closed after use.
pub fn get_window_exe_path(window_id: WindowIdentifier) -> Option<String> {
  use windows::Win32::Foundation::{CloseHandle, HWND};
  use windows::Win32::System::Threading::{
      OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
  };
  use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
  use windows::core::PWSTR;

  unsafe {
    let hwnd = HWND(window_id.as_u64() as *mut _);
    let mut process_id: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut process_id));

    if process_id == 0 {
      return None;
    }

    let process_handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id).ok()?;

    let mut path: [u16; 512] = [0; 512];
    let mut size = path.len() as u32;

    let result = QueryFullProcessImageNameW(
      process_handle,
      PROCESS_NAME_WIN32,
      PWSTR(path.as_mut_ptr()),
      &mut size,
    );

    let _ = CloseHandle(process_handle);

    if result.is_ok() && size > 0 {
      Some(String::from_utf16_lossy(&path[..size as usize]))
    } else {
      None
    }
  }
}

#[cfg(target_os = "linux")]
/// Gets the executable path of the process owning the specified window on Linux.
///
/// Uses X11 to get the process ID (PID) from the window property, then reads the symbolic link
/// from /proc/{pid}/exe to get the executable path. Returns the executable path, or `None` if
/// the X display can't be opened, the window doesn't exist, the PID property isn't set, or the
/// executable path can't be read.
///
/// # Safety
///
/// Uses unsafe X11 API calls. Properly cleans up resources by closing the display and freeing allocated memory.
pub fn get_window_exe_path(window_id: WindowIdentifier) -> Option<String> {
  use std::fs;
  use std::ptr;
  use x11::xlib::*;

  unsafe {
    let display = XOpenDisplay(ptr::null());
    if display.is_null() {
      return None;
    }

    let focus_window = window_id.as_u64();
    let atom_name = b"_NET_WM_PID\0".as_ptr() as *const i8;
    let pid_atom = XInternAtom(display, atom_name, 0);

    let mut actual_type: u64 = 0;
    let mut actual_format: i32 = 0;
    let mut nitems: u64 = 0;
    let mut bytes_after: u64 = 0;
    let mut prop: *mut u8 = ptr::null_mut();

    let status = XGetWindowProperty(
      display,
      focus_window,
      pid_atom,
      0,
      1,
      0,
      XA_CARDINAL,
      &mut actual_type,
      &mut actual_format,
      &mut nitems,
      &mut bytes_after,
      &mut prop,
    );

    let pid = if status == 0 && !prop.is_null() && nitems > 0 {
      let pid = *(prop as *const u32);
      XFree(prop as *mut _);
      pid
    } else {
      XCloseDisplay(display);
      return None;
    };

    XCloseDisplay(display);

    // Read the symlink from /proc/{pid}/exe
    let exe_path = format!("/proc/{}/exe", pid);
    fs::read_link(exe_path)
      .ok()
      .map(|p| p.to_string_lossy().to_string())
  }
}

#[cfg(target_os = "macos")]
/// Gets the executable path of the frontmost application on macOS.
///
/// Uses Cocoa/Objective-C APIs to get the frontmost application and its bundle path or executable URL.
/// Returns the executable path, or `None` if no frontmost application exists or the path can't be retrieved.
///
/// # Safety
///
/// Uses unsafe Objective-C message sending. Properly handles nil checks to prevent null pointer dereferences.
pub fn get_window_exe_path(_window_id: WindowIdentifier) -> Option<String> {
  use cocoa::base::{id, nil};
  use objc::{class, msg_send, sel, sel_impl};

  unsafe {
    let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
    let frontmost_app: id = msg_send![workspace, frontmostApplication];

    if frontmost_app == nil {
      return None;
    }

    let executable_url: id = msg_send![frontmost_app, executableURL];
    if executable_url == nil {
      return None;
    }

    let path: id = msg_send![executable_url, path];
    if path == nil {
      return None;
    }

    let path_ptr: *const i8 = msg_send![path, UTF8String];
    if path_ptr.is_null() {
      return None;
    }

    let c_str = std::ffi::CStr::from_ptr(path_ptr);
    Some(c_str.to_string_lossy().to_string())
  }
}
