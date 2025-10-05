use crate::WindowIdentifier;

#[cfg(target_os = "windows")]
/// Retrieves the size of the window on Windows.
///
/// This function uses the Windows API to get the window dimensions via `GetWindowRect`.
///
/// # Returns
///
/// * `Some((u32, u32))` - The window size (width, height) if successfully retrieved
/// * `None` - If the window handle is invalid, or the API call fails
///
/// # Safety
///
/// Uses unsafe Windows API calls with raw HWND handles.
pub fn get_window_size(window_id: WindowIdentifier) -> Option<(u32, u32)> {
    use std::mem::zeroed;
    use windows::Win32::Foundation::{HWND, RECT};
    use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;

    unsafe {
        let hwnd = HWND(window_id.as_u64() as *mut _);
        let mut rect: RECT = zeroed();

        if GetWindowRect(hwnd, &mut rect).is_ok() {
            let width = (rect.right - rect.left) as u32;
            let height = (rect.bottom - rect.top) as u32;
            Some((width, height))
        } else {
            None
        }
    }
}

#[cfg(target_os = "linux")]
/// Retrieves the size of the window on Linux.
///
/// This function uses X11 API to get the window dimensions.
///
/// # Returns
///
/// * `Some((u32, u32))` - The window size (width, height) if successfully retrieved
/// * `None` - If the window ID is invalid, or the API call fails
///
/// # Safety
///
/// Uses unsafe X11 API calls. Requires X11 display connection.
pub fn get_window_size(window_id: WindowIdentifier) -> Option<(u32, u32)> {
    use x11::xlib::{XCloseDisplay, XGetWindowAttributes, XOpenDisplay, XWindowAttributes};
    use std::mem::zeroed;
    use std::ptr;

    unsafe {
        let display = XOpenDisplay(ptr::null());
        if display.is_null() {
            return None;
        }

        let window = window_id.as_u64();
        let mut attributes: XWindowAttributes = zeroed();

        let result = XGetWindowAttributes(display, window, &mut attributes);
        XCloseDisplay(display);

        if result != 0 {
            Some((attributes.width as u32, attributes.height as u32))
        } else {
            None
        }
    }
}

#[cfg(target_os = "macos")]
/// Retrieves the size of the window on macOS.
///
/// This function uses Core Graphics API to get the window dimensions.
///
/// # Returns
///
/// * `Some((u32, u32))` - The window size (width, height) if successfully retrieved
/// * `None` - If the window ID is invalid, or the API call fails
pub fn get_window_size(window_id: WindowIdentifier) -> Option<(u32, u32)> {
    use core_graphics::window::{CGWindowListCopyWindowInfo, kCGWindowListOptionIncludingWindow};
    use core_foundation::array::CFArray;
    use core_foundation::base::TCFType;
    use core_foundation::dictionary::CFDictionary;
    use core_foundation::number::CFNumber;
    use core_foundation::string::CFString;

    unsafe {
        let window_list = CGWindowListCopyWindowInfo(
            kCGWindowListOptionIncludingWindow,
            window_id.as_u64() as u32
        );

        if window_list.is_null() {
            return None;
        }

        let array = CFArray::<CFDictionary>::wrap_under_create_rule(window_list as *const _);

        if array.len() == 0 {
            return None;
        }

        let window_info = array.get(0);
        let bounds_key = CFString::from_static_string("kCGWindowBounds");

        if let Some(bounds_dict) = window_info.find(&bounds_key) {
            let bounds_dict = bounds_dict as *const _ as *const CFDictionary;
            let bounds_dict = CFDictionary::wrap_under_get_rule(bounds_dict);

            let width_key = CFString::from_static_string("Width");
            let height_key = CFString::from_static_string("Height");

            let width = bounds_dict.find(&width_key)
                .and_then(|w| CFNumber::wrap_under_get_rule(w as *const _).to_i64())
                .map(|w| w as u32);

            let height = bounds_dict.find(&height_key)
                .and_then(|h| CFNumber::wrap_under_get_rule(h as *const _).to_i64())
                .map(|h| h as u32);

            match (width, height) {
                (Some(w), Some(h)) => Some((w, h)),
                _ => None,
            }
        } else {
            None
        }
    }
}