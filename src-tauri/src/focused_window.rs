//! Capture the foreground window's title at the moment the user starts
//! dictating, so Historial can show "you dictated this into <app>".
//!
//! - **Windows**: GetForegroundWindow + GetWindowTextW (native).
//! - **Linux / X11**: query `_NET_ACTIVE_WINDOW` on root, then `_NET_WM_NAME`
//!   (UTF-8) on the active window, with a `WM_NAME` (Latin-1) fallback.
//! - **Linux / Wayland**: `None` — the Wayland security model deliberately
//!   forbids an app from querying the focused window of another app. There
//!   is no portal for it and none is planned.
//! - **macOS**: stub (`None`) — will need NSWorkspace + AppKit bindings.

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

#[cfg(target_os = "macos")]
pub fn foreground_window_title() -> Option<String> {
    // TODO(macos): NSWorkspace shared => frontmostApplication => localizedName
    None
}

#[cfg(target_os = "linux")]
pub fn foreground_window_title() -> Option<String> {
    use crate::display_server::DisplayServer;

    // Wayland intentionally doesn't expose this — accept None.
    if DisplayServer::detect().is_wayland() {
        return None;
    }
    linux_x11_active_window_title()
}

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
pub fn foreground_window_title() -> Option<String> {
    None
}

/// Queries the X11 server for the active window's title. Opens a fresh
/// connection per call (cheap — a local Unix socket) rather than caching,
/// so a reconnect-after-server-restart is automatic. Any failure returns
/// `None`; the caller treats missing source_app as "unknown".
#[cfg(target_os = "linux")]
fn linux_x11_active_window_title() -> Option<String> {
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::{AtomEnum, ConnectionExt as _};

    let (conn, screen_num) = x11rb::connect(None).ok()?;
    let root = conn.setup().roots.get(screen_num)?.root;

    let net_active_window = conn
        .intern_atom(false, b"_NET_ACTIVE_WINDOW")
        .ok()?
        .reply()
        .ok()?
        .atom;
    let net_wm_name = conn
        .intern_atom(false, b"_NET_WM_NAME")
        .ok()?
        .reply()
        .ok()?
        .atom;
    let utf8_string = conn
        .intern_atom(false, b"UTF8_STRING")
        .ok()?
        .reply()
        .ok()?
        .atom;

    // Read the active window id (32-bit XID packed as little-endian 4 bytes).
    let active = conn
        .get_property(false, root, net_active_window, AtomEnum::WINDOW, 0, 1)
        .ok()?
        .reply()
        .ok()?;
    if active.value.len() < 4 {
        return None;
    }
    let win_id = u32::from_ne_bytes(active.value[..4].try_into().ok()?);
    if win_id == 0 {
        return None;
    }

    // Try EWMH _NET_WM_NAME (UTF-8) first, then ICCCM WM_NAME (Latin-1).
    let net_name = conn
        .get_property(false, win_id, net_wm_name, utf8_string, 0, u32::MAX)
        .ok()?
        .reply()
        .ok()?;
    if !net_name.value.is_empty() {
        if let Ok(s) = String::from_utf8(net_name.value) {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    let wm_name = conn
        .get_property(
            false,
            win_id,
            AtomEnum::WM_NAME,
            AtomEnum::STRING,
            0,
            u32::MAX,
        )
        .ok()?
        .reply()
        .ok()?;
    if wm_name.value.is_empty() {
        return None;
    }
    let s = String::from_utf8_lossy(&wm_name.value).into_owned();
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
