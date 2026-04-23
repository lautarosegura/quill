//! HotkeyManager — low-level global keyboard listening for push-to-talk.
//!
//! Uses `rdev::grab` (not `listen`) so we can *suppress* events as they flow
//! through the OS. That matters for two cases:
//!
//! 1. **Modifier-only chords on Windows** (e.g. Ctrl + Win, the default
//!    `Keybind` on Windows). If Windows sees a Win-down, it later interprets
//!    accompanying keys (even ones our injector sends later) as Win+X
//!    shortcuts — notably `Win+Ctrl+V` opens the Sound output picker. We
//!    must eat every Meta event (initial press, auto-repeats, release)
//!    while Ctrl is held so Windows' keyboard state stays "Meta=up".
//! 2. **Trigger-key chords** (e.g. Ctrl + Shift + Space). We eat the trigger
//!    so no stray character leaks into the focused text field.
//!
//! Symmetry is critical: if we suppressed a press, we MUST suppress the
//! matching release, or the OS ends up with a release-without-press that
//! can trigger phantom behavior. `hiding_meta` / `hiding_trigger` track that.
//!
//! The hotkey is read from a shared `Arc<RwLock<Config>>` on every event,
//! so runtime changes from the Settings UI apply to the next keystroke
//! without restarting the app.
//!
//! Emits paired `Pressed`/`Released` events: the orchestrator filters short
//! taps via `held_ms`. Filtering *here* would break the paired-event
//! invariant (an orphan `Pressed` would leave the orchestrator wedged).

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use rdev::{grab, Event, EventType, Key};
use tokio::sync::{mpsc, RwLock};

use crate::config::Config;
use crate::focused_window;
use crate::types::{Keybind, Modifier};

#[derive(Debug, Clone)]
pub enum HotkeyEvent {
    /// User started the chord. `source_app` is the title of the foreground
    /// window at that moment (captured on the hook thread, cheap).
    Pressed {
        source_app: Option<String>,
    },
    Released {
        held_ms: u64,
    },
    /// User pressed Escape — the orchestrator cancels an active (locked)
    /// session if there is one, otherwise ignores.
    CancelRequested,
    /// User pressed Alt+Shift+Z — re-paste the last successful transcription
    /// into the focused app.
    ReinjectLast,
}

pub struct HotkeyManager {
    _thread: thread::JoinHandle<()>,
}

impl HotkeyManager {
    pub fn start(config: Arc<RwLock<Config>>) -> (Self, mpsc::UnboundedReceiver<HotkeyEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let state = Arc::new(Mutex::new(HotkeyState::default()));

        let thread = thread::spawn(move || {
            let cb = move |event: Event| -> Option<Event> {
                // blocking_read is safe here: rdev's listener thread is not a
                // tokio runtime worker. Re-read per event so Settings changes
                // apply on the next keystroke without restart.
                let hotkey = config.blocking_read().hotkey.clone();
                handle_event(event, &hotkey, &state, &tx)
            };
            if let Err(e) = grab(cb) {
                log::error!("rdev::grab failed to install or crashed: {e:?}");
            }
        });

        (Self { _thread: thread }, rx)
    }
}

#[derive(Default)]
struct HotkeyState {
    ctrl: bool,
    shift: bool,
    alt: bool,
    meta: bool,
    /// True while the chord is active (between Pressed and Released emits).
    chord_active: bool,
    pressed_at: Option<Instant>,
    /// True if we have suppressed at least one Meta press; the matching
    /// release (and any intervening auto-repeats) must also be suppressed.
    hiding_meta: bool,
    /// Same idea for the trigger key of a trigger-based chord.
    hiding_trigger: bool,
    /// True if we just suppressed a Z press from an Alt+Shift+Z chord — the
    /// matching keyup must also be eaten so it doesn't leak to the focused app.
    hiding_repaste_z: bool,
}

/// Per-event dispatch. Returns `Some(event)` to let the event pass through
/// to the rest of the OS, `None` to suppress it.
fn handle_event(
    event: Event,
    hotkey: &Keybind,
    state: &Arc<Mutex<HotkeyState>>,
    tx: &mpsc::UnboundedSender<HotkeyEvent>,
) -> Option<Event> {
    let mut st = state.lock().unwrap();

    match event.event_type {
        EventType::KeyPress(k) => {
            // Update modifier state FIRST so `modifiers_held` reflects the
            // post-event state.
            update_modifier(&mut st, k, true);

            // Global helper chord: Escape — orchestrator decides whether to
            // cancel an in-flight session. No suppression: pass Escape through
            // so text fields / modals still get their normal dismiss behavior.
            if matches!(k, Key::Escape) {
                let _ = tx.send(HotkeyEvent::CancelRequested);
            }

            // Global helper chord: Alt + Shift + Z re-pastes the last success.
            // Suppress the Z (and its matching keyup) so no stray "z" leaks
            // into the focused app when the user is just asking for a paste.
            if matches!(k, Key::KeyZ) {
                log::info!(
                    "Z press observed: alt={} shift={} ctrl={} meta={}",
                    st.alt,
                    st.shift,
                    st.ctrl,
                    st.meta
                );
            }
            if matches!(k, Key::KeyZ) && st.alt && st.shift && !st.ctrl && !st.meta {
                log::info!("Alt+Shift+Z chord detected — emitting ReinjectLast");
                let _ = tx.send(HotkeyEvent::ReinjectLast);
                st.hiding_repaste_z = true;
                return None;
            }

            // Emit Pressed on the transition *into* chord_active.
            if should_activate_chord(&st, &k, hotkey) && !st.chord_active {
                st.chord_active = true;
                st.pressed_at = Some(Instant::now());
                // Snapshot foreground window at the moment the user pressed —
                // this is the app they were typing into. Doing it here (on the
                // rdev hook thread) is cheap and avoids racing against focus
                // changes caused by our own downstream work.
                let source_app = focused_window::foreground_window_title();
                let _ = tx.send(HotkeyEvent::Pressed { source_app });
            }

            // Suppression is decided *independent* of chord_active so that
            // auto-repeat Meta presses (which fire many times per second
            // while the key is physically held) all get eaten, not just the
            // first one.
            if should_suppress_press(&st, &k, hotkey) {
                if matches!(k, Key::MetaLeft | Key::MetaRight) {
                    st.hiding_meta = true;
                }
                if let Some(trigger) = &hotkey.key {
                    if rdev_key_matches(&k, trigger) {
                        st.hiding_trigger = true;
                    }
                }
                return None;
            }
            Some(event)
        }
        EventType::KeyRelease(k) => {
            // Emit Released + update modifier state. The emit has to happen
            // before we update_modifier because `should_release_chord`
            // doesn't consult st (it only cares about which key was released).
            if st.chord_active && should_release_chord(&k, hotkey) {
                st.chord_active = false;
                let held_ms = st
                    .pressed_at
                    .take()
                    .map(|t| t.elapsed().as_millis() as u64)
                    .unwrap_or(0);
                let _ = tx.send(HotkeyEvent::Released { held_ms });
            }

            update_modifier(&mut st, k, false);

            // Symmetry: if we hid the press for this key, we MUST hide the
            // release. Otherwise the OS sees a release-without-press which
            // on Windows can trigger phantom shortcut behavior.
            let is_meta = matches!(k, Key::MetaLeft | Key::MetaRight);
            if is_meta && st.hiding_meta {
                st.hiding_meta = false;
                return None;
            }
            if let Some(trigger) = &hotkey.key {
                if rdev_key_matches(&k, trigger) && st.hiding_trigger {
                    st.hiding_trigger = false;
                    return None;
                }
            }

            // Repaste chord trailing-Z suppression.
            if matches!(k, Key::KeyZ) && st.hiding_repaste_z {
                st.hiding_repaste_z = false;
                return None;
            }

            Some(event)
        }
        _ => Some(event),
    }
}

fn update_modifier(st: &mut HotkeyState, k: Key, pressed: bool) {
    match k {
        Key::ControlLeft | Key::ControlRight => st.ctrl = pressed,
        Key::ShiftLeft | Key::ShiftRight => st.shift = pressed,
        Key::Alt | Key::AltGr => st.alt = pressed,
        Key::MetaLeft | Key::MetaRight => st.meta = pressed,
        _ => {}
    }
}

fn modifiers_held(st: &HotkeyState, required: &[Modifier]) -> bool {
    required.iter().all(|m| match m {
        Modifier::Ctrl => st.ctrl,
        Modifier::Shift => st.shift,
        Modifier::Alt => st.alt,
        Modifier::Meta => st.meta,
    })
}

/// Any required modifier *other than Meta* is currently held. Used to decide
/// whether an incoming Meta press is part of our chord and should be eaten.
fn other_required_modifier_held(st: &HotkeyState, hotkey: &Keybind) -> bool {
    hotkey.modifiers.iter().any(|m| match m {
        Modifier::Ctrl => st.ctrl,
        Modifier::Shift => st.shift,
        Modifier::Alt => st.alt,
        Modifier::Meta => false,
    })
}

/// Given the *just-pressed* key and current modifier state, should we enter
/// the chord right now?
fn should_activate_chord(st: &HotkeyState, k: &Key, hotkey: &Keybind) -> bool {
    if !modifiers_held(st, &hotkey.modifiers) {
        return false;
    }
    match &hotkey.key {
        Some(trigger) => rdev_key_matches(k, trigger),
        None => is_required_modifier_key(k, &hotkey.modifiers),
    }
}

/// Given a *just-released* key, does it break the chord?
fn should_release_chord(k: &Key, hotkey: &Keybind) -> bool {
    match &hotkey.key {
        Some(trigger) => rdev_key_matches(k, trigger),
        None => is_required_modifier_key(k, &hotkey.modifiers),
    }
}

/// Should we consume this keypress before it reaches the rest of the OS?
///
/// - Trigger chord: yes for the trigger key (so no stray letter leaks).
/// - Modifier-only chord with Meta: yes for any Meta event while another
///   required modifier is held. Runs on every auto-repeat, not just first.
fn should_suppress_press(st: &HotkeyState, k: &Key, hotkey: &Keybind) -> bool {
    match &hotkey.key {
        Some(trigger) => rdev_key_matches(k, trigger),
        None => {
            matches!(k, Key::MetaLeft | Key::MetaRight)
                && hotkey.modifiers.contains(&Modifier::Meta)
                && other_required_modifier_held(st, hotkey)
        }
    }
}

fn is_required_modifier_key(k: &Key, required: &[Modifier]) -> bool {
    let m = match k {
        Key::ControlLeft | Key::ControlRight => Some(Modifier::Ctrl),
        Key::ShiftLeft | Key::ShiftRight => Some(Modifier::Shift),
        Key::Alt | Key::AltGr => Some(Modifier::Alt),
        Key::MetaLeft | Key::MetaRight => Some(Modifier::Meta),
        _ => None,
    };
    match m {
        Some(m) => required.contains(&m),
        None => false,
    }
}

/// Maps a user-facing key name (e.g. "Space", "A", "F1") to rdev's Key enum.
fn rdev_key_matches(k: &Key, name: &str) -> bool {
    let candidate = match name.to_lowercase().as_str() {
        "space" => Some(Key::Space),
        "enter" | "return" => Some(Key::Return),
        "tab" => Some(Key::Tab),
        "escape" | "esc" => Some(Key::Escape),
        _ => None,
    };
    if let Some(target) = candidate {
        return *k == target;
    }
    if name.len() == 1 {
        let c = name.chars().next().unwrap().to_ascii_uppercase();
        if c.is_ascii_uppercase() {
            return format!("{k:?}") == format!("Key{c}");
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trigger_kb() -> Keybind {
        Keybind {
            modifiers: vec![Modifier::Ctrl, Modifier::Shift],
            key: Some("Space".into()),
        }
    }

    fn modifier_only_kb() -> Keybind {
        Keybind {
            modifiers: vec![Modifier::Ctrl, Modifier::Meta],
            key: None,
        }
    }

    #[test]
    fn trigger_chord_activates_on_trigger_press_with_mods_held() {
        let st = HotkeyState {
            ctrl: true,
            shift: true,
            ..Default::default()
        };
        assert!(should_activate_chord(&st, &Key::Space, &trigger_kb()));
        assert!(should_release_chord(&Key::Space, &trigger_kb()));
    }

    #[test]
    fn trigger_chord_does_not_activate_without_required_mods() {
        let st = HotkeyState {
            ctrl: true, // shift missing
            ..Default::default()
        };
        assert!(!should_activate_chord(&st, &Key::Space, &trigger_kb()));
    }

    #[test]
    fn modifier_only_chord_activates_when_last_mod_goes_down() {
        let st = HotkeyState {
            ctrl: true,
            meta: true,
            ..Default::default()
        };
        assert!(should_activate_chord(
            &st,
            &Key::MetaLeft,
            &modifier_only_kb()
        ));
    }

    #[test]
    fn modifier_only_chord_ignores_unrelated_presses() {
        let st = HotkeyState {
            ctrl: true,
            meta: true,
            ..Default::default()
        };
        assert!(!should_activate_chord(&st, &Key::KeyA, &modifier_only_kb()));
    }

    #[test]
    fn modifier_only_chord_releases_on_either_modifier_up() {
        assert!(should_release_chord(&Key::MetaLeft, &modifier_only_kb()));
        assert!(should_release_chord(&Key::ControlLeft, &modifier_only_kb()));
        assert!(!should_release_chord(&Key::ShiftLeft, &modifier_only_kb()));
    }

    #[test]
    fn suppresses_trigger_press_for_trigger_chord() {
        let st = HotkeyState::default();
        assert!(should_suppress_press(&st, &Key::Space, &trigger_kb()));
        // Modifiers themselves pass through.
        assert!(!should_suppress_press(
            &st,
            &Key::ControlLeft,
            &trigger_kb()
        ));
    }

    #[test]
    fn suppresses_meta_press_whenever_other_required_mod_is_held() {
        // Ctrl held (our other required mod) → Meta press must be eaten,
        // whether this is the initial press or an auto-repeat.
        let st = HotkeyState {
            ctrl: true,
            ..Default::default()
        };
        assert!(should_suppress_press(
            &st,
            &Key::MetaLeft,
            &modifier_only_kb()
        ));
        assert!(should_suppress_press(
            &st,
            &Key::MetaRight,
            &modifier_only_kb()
        ));
    }

    #[test]
    fn does_not_suppress_meta_press_without_other_required_mod() {
        // Ctrl NOT held → user is pressing Win alone, not our chord. Let it
        // through so Start menu, Win+D, etc. still work.
        let st = HotkeyState::default();
        assert!(!should_suppress_press(
            &st,
            &Key::MetaLeft,
            &modifier_only_kb()
        ));
    }

    #[test]
    fn does_not_suppress_ctrl_in_modifier_only_chord() {
        // Ctrl doesn't have a Start-menu-equivalent side effect, so we let
        // it through even though it's part of the chord.
        let st = HotkeyState {
            meta: true,
            ..Default::default()
        };
        assert!(!should_suppress_press(
            &st,
            &Key::ControlLeft,
            &modifier_only_kb()
        ));
        assert!(!should_suppress_press(
            &st,
            &Key::ControlRight,
            &modifier_only_kb()
        ));
    }

    #[test]
    fn does_not_suppress_meta_when_meta_not_in_chord() {
        let st = HotkeyState {
            ctrl: true,
            shift: true,
            ..Default::default()
        };
        assert!(!should_suppress_press(&st, &Key::MetaLeft, &trigger_kb()));
    }

    #[test]
    fn rdev_key_matches_handles_named_keys() {
        assert!(rdev_key_matches(&Key::Space, "Space"));
        assert!(rdev_key_matches(&Key::Space, "space"));
        assert!(rdev_key_matches(&Key::Return, "Enter"));
        assert!(rdev_key_matches(&Key::Escape, "Esc"));
        assert!(!rdev_key_matches(&Key::Space, "Tab"));
    }

    #[test]
    fn rdev_key_matches_handles_single_letters() {
        assert!(rdev_key_matches(&Key::KeyA, "A"));
        assert!(rdev_key_matches(&Key::KeyZ, "z"));
        assert!(!rdev_key_matches(&Key::KeyA, "B"));
    }
}
