use crate::window::{WindowIdentifier, get_current_window_identifier};

#[cfg(target_os = "windows")]
/// Retrieves the title of the specified window on Windows.
///
/// This function uses the Windows API to get the window handle
/// and retrieves its title text.
///
/// # Returns
///
/// * `Some(String)` - The window title if successfully retrieved
/// * `None` - If the window doesn't exist or the title is empty
///
/// # Safety
///
/// Uses unsafe Windows API calls. The buffer size is limited to 512 characters.
pub fn get_window_title(window_id: WindowIdentifier) -> Option<String> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::GetWindowTextW;

    unsafe {
        let hwnd = HWND(window_id.as_u64() as *mut _);
        let mut title: [u16; 512] = [0; 512];
        let len = GetWindowTextW(hwnd, &mut title);

        if len > 0 {
            Some(String::from_utf16_lossy(&title[..len as usize]))
        } else {
            None
        }
    }
}

#[cfg(target_os = "linux")]
/// Retrieves the title of the specified window on Linux.
///
/// This function uses X11 (Xlib) to get the window by its identifier
/// and fetches its name property.
///
/// # Returns
///
/// * `Some(String)` - The window title if successfully retrieved
/// * `None` - If the X display cannot be opened, the window doesn't exist,
///   or the window has no name
///
/// # Safety
///
/// Uses unsafe X11 API calls. Properly cleans up resources by closing
/// the display and freeing allocated memory.
pub fn get_window_title(window_id: WindowIdentifier) -> Option<String> {
    use std::ffi::CStr;
    use std::ptr;
    use x11::xlib::*;

    unsafe {
        let display = XOpenDisplay(ptr::null());
        if display.is_null() {
            return None;
        }

        let focus_window = window_id.as_u64();
        let mut name: *mut i8 = ptr::null_mut();
        let status = XFetchName(display, focus_window, &mut name);

        let title = if status != 0 && !name.is_null() {
            let c_str = CStr::from_ptr(name);
            let title = c_str.to_string_lossy().to_string();
            XFree(name as *mut _);
            Some(title)
        } else {
            None
        };

        XCloseDisplay(display);
        title
    }
}

#[cfg(target_os = "macos")]
/// Retrieves the title of the specified window on macOS.
///
/// This function uses Cocoa/Objective-C APIs to get the window information.
/// Note: On macOS, if the window_id doesn't correspond to an actual window,
/// this returns the application name of the frontmost application.
///
/// # Returns
///
/// * `Some(String)` - The window title or application name if successfully retrieved
/// * `None` - If there is no window or application or the name cannot be retrieved
///
/// # Safety
///
/// Uses unsafe Objective-C message sending. Properly handles nil checks
/// to prevent null pointer dereferences.
pub fn get_window_title(_window_id: WindowIdentifier) -> Option<String> {
    use cocoa::base::{id, nil};
    use objc::{class, msg_send, sel, sel_impl};

    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let frontmost_app: id = msg_send![workspace, frontmostApplication];

        if frontmost_app == nil {
            return None;
        }

        let localized_name: id = msg_send![frontmost_app, localizedName];
        if localized_name == nil {
            return None;
        }

        let name_ptr: *const i8 = msg_send![localized_name, UTF8String];
        if name_ptr.is_null() {
            return None;
        }

        let c_str = std::ffi::CStr::from_ptr(name_ptr);
        Some(c_str.to_string_lossy().to_string())
    }
}
