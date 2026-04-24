//! Wayland auto-paste via XDG RemoteDesktop portal + libei.
//!
//! Called from `TextInjector::inject` on Wayland, AFTER
//! `wayland_clipboard_hold` has put the text on the clipboard. This module
//! asks the compositor (via the XDG `RemoteDesktop` portal) for permission
//! to synthesize keyboard input, then emits `Ctrl+V` via libei so the
//! focused app pastes the dictation automatically.
//!
//! ### First-run consent
//!
//! On the very first call per app launch the compositor pops a system
//! dialog (GNOME / KDE) asking the user to approve "Quill can emulate
//! keyboard input". We use `PersistMode::ExplicitlyRevoked` and cache the
//! resulting restore token in a process-wide static so subsequent calls
//! within the same session skip the dialog. v0.3 will persist the token
//! in Config so the dialog only appears once in the app's lifetime.
//!
//! ### Compositor support
//!
//! Works on GNOME 48+, KDE Plasma 6+, and any other compositor that
//! implements the RemoteDesktop XDG portal. Sway / wlroots / older
//! compositors return an error from this module — the caller falls back
//! to clipboard-only (user manually presses Ctrl+V).
//!
//! ### Reference
//!
//! The libei event-loop sequence mirrors the upstream
//! `ids1024/reis/examples/type-text.rs` template — that's the ground
//! truth for the handshake / seat-bind / device-activation dance.

#![cfg(target_os = "linux")]

use std::collections::HashMap;
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixStream;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use ashpd::desktop::remote_desktop::{
    ConnectToEISOptions, DeviceType, RemoteDesktop, SelectDevicesOptions, StartOptions,
};
use ashpd::desktop::{CreateSessionOptions, PersistMode};
use calloop::generic::Generic;
use calloop::{EventLoop, Interest, Mode, PostAction};
use enumflags2::BitFlags;
use reis::{ei, PendingRequestResult};

use crate::error::QuillError;

/// evdev keycodes (what libei wants — not xkb codes).
/// Sourced from `/usr/include/linux/input-event-codes.h`.
const KEY_LEFTCTRL: u32 = 29;
const KEY_V: u32 = 47;

/// How long we'll wait for the libei handshake + seat-bind + keyboard
/// device activation before giving up and falling back to clipboard-only.
/// 8 s is generous — on a working compositor the whole dance finishes in
/// well under a second; this just guards against a hung portal dialog.
const PASTE_TIMEOUT: Duration = Duration::from_secs(8);

/// Process-wide cache of the RemoteDesktop session restore token. The
/// first successful `start()` populates this; subsequent calls pass it
/// back to `select_devices` so the compositor recognises us as
/// already-approved and skips the consent dialog.
static RESTORE_TOKEN: Mutex<Option<String>> = Mutex::new(None);

/// Synthesize Ctrl+V in the focused application. Assumes the clipboard
/// already holds the payload. Returns `Ok(())` on successful delivery;
/// all failure paths return `Err`, and the caller is expected to fall
/// back to clipboard-only UX (user presses Ctrl+V themselves).
pub async fn paste_ctrl_v() -> Result<(), QuillError> {
    let prior_token = RESTORE_TOKEN.lock().unwrap().clone();
    let (fd, new_token) = setup_session(prior_token).await?;
    if let Some(t) = new_token {
        *RESTORE_TOKEN.lock().unwrap() = Some(t);
    }
    // The libei protocol loop is synchronous calloop — run it on a
    // tokio blocking thread so we don't freeze the async runtime while
    // waiting for the compositor.
    tokio::task::spawn_blocking(move || emit_ctrl_v(fd))
        .await
        .map_err(|e| QuillError::Injection(format!("libei task join: {e}")))??;
    Ok(())
}

/// Stands up a fresh RemoteDesktop session, selects the keyboard device,
/// prompts the user (first run only) via the portal dialog, and returns
/// a libei socket fd plus any issued restore token.
async fn setup_session(
    prior_token: Option<String>,
) -> Result<(OwnedFd, Option<String>), QuillError> {
    let proxy = RemoteDesktop::new()
        .await
        .map_err(|e| QuillError::Injection(format!("RemoteDesktop::new: {e}")))?;
    let session = proxy
        .create_session(CreateSessionOptions::default())
        .await
        .map_err(|e| QuillError::Injection(format!("create_session: {e}")))?;

    let mut select_opts = SelectDevicesOptions::default()
        .set_devices(BitFlags::from(DeviceType::Keyboard))
        .set_persist_mode(PersistMode::ExplicitlyRevoked);
    if let Some(token) = prior_token {
        select_opts = select_opts.set_restore_token(token);
    }
    proxy
        .select_devices(&session, select_opts)
        .await
        .map_err(|e| QuillError::Injection(format!("select_devices: {e}")))?;

    // The first call on a given app launch pops a compositor dialog; on
    // subsequent calls (with the cached restore token) the portal
    // returns immediately.
    let start_response = proxy
        .start(&session, None, StartOptions::default())
        .await
        .map_err(|e| QuillError::Injection(format!("start: {e}")))?;
    let selected = start_response
        .response()
        .map_err(|e| QuillError::Injection(format!("start response: {e}")))?;
    let new_token = selected.restore_token().map(String::from);

    let fd = proxy
        .connect_to_eis(&session, ConnectToEISOptions::default())
        .await
        .map_err(|e| QuillError::Injection(format!("connect_to_eis: {e}")))?;
    Ok((fd, new_token))
}

fn emit_ctrl_v(fd: OwnedFd) -> Result<(), QuillError> {
    let stream = UnixStream::from(fd);
    stream
        .set_nonblocking(true)
        .map_err(|e| QuillError::Injection(format!("set_nonblocking: {e}")))?;
    let context = ei::Context::new(stream)
        .map_err(|e| QuillError::Injection(format!("ei::Context::new: {e}")))?;
    let _handshake = context.handshake();
    let _ = context.flush();

    let mut state = State {
        seats: HashMap::new(),
        devices: HashMap::new(),
        sequence: 0,
        last_serial: u32::MAX,
        emitted: false,
    };

    let mut event_loop: EventLoop<State> = EventLoop::try_new()
        .map_err(|e| QuillError::Injection(format!("calloop EventLoop: {e}")))?;
    let handle = event_loop.handle();
    let source = Generic::new(context, Interest::READ, Mode::Level);
    handle
        .insert_source(source, |_readiness, context, state: &mut State| {
            handle_readable(unsafe { context.get_mut() }, state)
        })
        .map_err(|e| QuillError::Injection(format!("calloop insert: {e}")))?;

    let deadline = Instant::now() + PASTE_TIMEOUT;
    while !state.emitted {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or_else(|| {
                QuillError::Injection(
                    "libei paste timed out before reaching keyboard device".into(),
                )
            })?;
        event_loop
            .dispatch(Some(remaining), &mut state)
            .map_err(|e| QuillError::Injection(format!("calloop dispatch: {e}")))?;
    }
    Ok(())
}

struct State {
    seats: HashMap<ei::Seat, SeatData>,
    devices: HashMap<ei::Device, DeviceData>,
    sequence: u32,
    last_serial: u32,
    emitted: bool,
}

#[derive(Default)]
struct SeatData {
    capabilities: HashMap<String, u64>,
}

#[derive(Default)]
struct DeviceData {
    interfaces: HashMap<String, reis::Object>,
}

impl DeviceData {
    fn interface<T: reis::Interface>(&self) -> Option<T> {
        self.interfaces.get(T::NAME)?.clone().downcast()
    }
}

fn handle_readable(context: &mut ei::Context, state: &mut State) -> std::io::Result<PostAction> {
    if context.read().is_err() {
        return Ok(PostAction::Remove);
    }

    while let Some(result) = context.pending_event() {
        let request = match result {
            PendingRequestResult::Request(request) => request,
            PendingRequestResult::ParseError(_) | PendingRequestResult::InvalidObject(_) => {
                continue;
            }
        };
        match request {
            ei::Event::Handshake(handshake, req) => match req {
                ei::handshake::Event::HandshakeVersion { .. } => {
                    handshake.handshake_version(1);
                    handshake.name("quill");
                    handshake.context_type(ei::handshake::ContextType::Sender);
                    for (iface, v) in [
                        ("ei_callback", 1),
                        ("ei_connection", 1),
                        ("ei_seat", 1),
                        ("ei_device", 1),
                        ("ei_pingpong", 1),
                        ("ei_keyboard", 1),
                    ] {
                        handshake.interface_version(iface, v);
                    }
                    handshake.finish();
                }
                ei::handshake::Event::Connection { serial, .. } => {
                    state.last_serial = serial;
                }
                _ => {}
            },
            ei::Event::Connection(_, req) => match req {
                ei::connection::Event::Seat { seat } => {
                    state.seats.insert(seat, SeatData::default());
                }
                ei::connection::Event::Ping { ping } => {
                    // Responding to pings keeps the compositor from
                    // tearing us down for unresponsiveness.
                    ping.done(0);
                }
                _ => {}
            },
            ei::Event::Seat(seat, req) => {
                let data = state.seats.entry(seat.clone()).or_default();
                match req {
                    ei::seat::Event::Capability { mask, interface } => {
                        data.capabilities.insert(interface, mask);
                    }
                    ei::seat::Event::Done => {
                        if let Some(cap) = data.capabilities.get("ei_keyboard").copied() {
                            seat.bind(cap);
                        }
                    }
                    ei::seat::Event::Device { device } => {
                        state.devices.insert(device, DeviceData::default());
                    }
                    _ => {}
                }
            }
            ei::Event::Device(device, req) => {
                let data = state.devices.entry(device.clone()).or_default();
                match req {
                    ei::device::Event::Interface { object } => {
                        data.interfaces
                            .insert(object.interface().to_owned(), object);
                    }
                    ei::device::Event::Done => {
                        if let Some(keyboard) = data.interface::<ei::Keyboard>() {
                            // The emit sequence is verbatim from
                            // upstream's type-text example: start →
                            // press/release → frame → stop. The frame()
                            // call is what actually submits the events;
                            // without it the compositor drops them.
                            device.start_emulating(state.sequence, state.last_serial);
                            state.sequence += 1;
                            keyboard.key(KEY_LEFTCTRL, ei::keyboard::KeyState::Press);
                            keyboard.key(KEY_V, ei::keyboard::KeyState::Press);
                            keyboard.key(KEY_V, ei::keyboard::KeyState::Released);
                            keyboard.key(KEY_LEFTCTRL, ei::keyboard::KeyState::Released);
                            let now_us = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_micros() as u64)
                                .unwrap_or(0);
                            device.frame(state.last_serial, now_us);
                            device.stop_emulating(state.last_serial);
                            state.emitted = true;
                        }
                    }
                    ei::device::Event::Resumed { serial } => {
                        state.last_serial = serial;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    let _ = context.flush();
    Ok(PostAction::Continue)
}
