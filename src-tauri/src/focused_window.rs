//! Capture the foreground window's title at the moment the user starts
//! dictating, so Historial can show "you dictated this into <app>".
//!
//! Windows only for now — macOS needs NSWorkspace + AppKit bindings, which
//! we'll add when we cross-compile for Darwin.

/// Returns the title of the focused window (or `None` if no window is focused
/// or the OS call fails). Cheap synchronous call; safe to run from the rdev
/// hook thread.
#[cfg(windows)]
pub fn foreground_window_title() -> Option<String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW,
    };
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return None;
        }
        let len = GetWindowTextLengthW(hwnd);
        if len <= 0 {
            return None;
        }
        let mut buf: Vec<u16> = vec![0; (len as usize) + 1];
        let copied = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
        if copied <= 0 {
            return None;
        }
        let s = String::from_utf16_lossy(&buf[..copied as usize]);
        let trimmed = s.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }
}

#[cfg(not(windows))]
pub fn foreground_window_title() -> Option<String> {
    // TODO(macos): NSWorkspace shared => frontmostApplication => localizedName
    None
}
