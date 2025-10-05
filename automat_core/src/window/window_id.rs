/// Cross-platform window identifier type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowIdentifier(u64);

impl WindowIdentifier {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

#[cfg(target_os = "windows")]
/// Retrieves the identifier of the currently focused window on Windows.
///
/// # Returns
///
/// * `Some(WindowIdentifier)` - The window identifier (HWND cast to u64)
/// * `None` - If there is no foreground window
pub fn get_current_window_identifier() -> Option<WindowIdentifier> {
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0 == std::ptr::null_mut() {
            None
        } else {
            Some(WindowIdentifier::new(hwnd.0 as u64))
        }
    }
}

#[cfg(target_os = "linux")]
/// Retrieves the identifier of the currently focused window on Linux.
///
/// # Returns
///
/// * `Some(WindowIdentifier)` - The window identifier (X11 Window ID)
/// * `None` - If the X display cannot be opened or there is no focused window
pub fn get_current_window_identifier() -> Option<WindowIdentifier> {
    use std::ptr;
    use x11::xlib::*;

    unsafe {
        let display = XOpenDisplay(ptr::null());
        if display.is_null() {
            return None;
        }

        let mut focus_window: Window = 0;
        let mut revert_to: i32 = 0;
        XGetInputFocus(display, &mut focus_window, &mut revert_to);

        XCloseDisplay(display);

        if focus_window == 0 {
            None
        } else {
            Some(WindowIdentifier::new(focus_window))
        }
    }
}

#[cfg(target_os = "macos")]
/// Retrieves the identifier of the currently focused application on macOS.
///
/// Note: On macOS, this returns a process identifier (PID) since there's no
/// direct window handle equivalent like on Windows or X11.
///
/// # Returns
///
/// * `Some(WindowIdentifier)` - The application process ID
/// * `None` - If there is no frontmost application
pub fn get_current_window_identifier() -> Option<WindowIdentifier> {
    use cocoa::base::{id, nil};
    use objc::{class, msg_send, sel, sel_impl};

    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let frontmost_app: id = msg_send![workspace, frontmostApplication];

        if frontmost_app == nil {
            return None;
        }

        let pid: i32 = msg_send![frontmost_app, processIdentifier];
        Some(WindowIdentifier::new(pid as u64))
    }
}
