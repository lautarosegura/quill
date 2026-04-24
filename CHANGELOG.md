# Changelog

All notable changes to Quill are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and Quill uses
[semantic versioning](https://semver.org/). Pre-1.0 releases may include
breaking changes between minor versions.

## [Unreleased]

## [0.2.1] — 2026-04-24

Linux bug-fix release from first real-world testing of the v0.2.0 `.deb`
on Wayland. Several paths were silently broken; v0.2.1 closes the most
important ones.

### Fixed

- **Wizard now persists `wizard_version=1` on finish** — the `finish_wizard`
  Tauri command writes the marker directly, idempotently. Works even if
  the frontend's `applyDraftToRuntime` racing / stale-config-store path
  silently dropped the write (which it did on GNOME Wayland in v0.2.0).
- **Wayland clipboard was empty after dictation** (even manual Ctrl+V
  pasted nothing). Root cause: `arboard` auto-picks its X11 backend
  when `$DISPLAY` is set, which is always the case on GNOME Wayland
  (XWayland), so we were writing to the wrong selection. Fix: talk to
  the Wayland compositor directly via `wl-clipboard-rs`, held alive by
  a detached thread with `ServeRequests::Unlimited` until another
  client takes selection ownership.

### Added

- **Auto-paste on Wayland via libei** (XDG `RemoteDesktop` portal +
  `reis`). After transcription Quill synthesizes Ctrl+V itself — same UX
  as Windows/X11, no more "press Ctrl+V to paste" toast. First paste
  triggers a one-time RemoteDesktop consent dialog on GNOME/KDE; restore
  token is cached in-process so subsequent pastes are silent. Falls
  back to clipboard-only if the portal is denied or unavailable.
- **Dual Wayland hotkey backend** — XDG `GlobalShortcuts` portal
  primary, `rdev::listen` via `/dev/input/event*` as a fallback. Covers
  compositors without the portal (GNOME 46/47, Sway, wlroots, older
  Hyprland). Requires the user to be in the `input` group for the
  fallback; documented in the README compatibility table.
- **Dense logging in `wayland_backend::run`** — each portal stage
  (proxy connect, session create, bind_shortcuts, stream subscribe)
  and each `Activated`/`Deactivated` signal now logs at INFO, so
  debugging hotkey timing on different compositors is straightforward.
- **`libxcb1`, `libwayland-client0`, `libxdo3`** added to `.deb`
  depends. `libxdo3` was the specific runtime library missing on v0.2.0
  installs (enigo links against it for the X11 paste path).

### Known limitations (carried over)

- Wayland auto-paste consent token isn't persisted to disk yet — the
  first paste of each app launch still prompts. v0.3 fixes this by
  saving the restore_token in Config.
- `libayatana-appindicator` prints a "deprecated" warning on stderr.
  Cosmetic; waiting on Tauri upstream to migrate to `-glib-1`.

## [0.2.0] — 2026-04-23

Linux support lands: both X11 and Wayland sessions, shipped as `.deb` and
`.AppImage` installers alongside the existing Windows NSIS + MSI.

### Added

- **Linux X11**: full feature parity with Windows — push-to-talk via
  `rdev::grab`, focused-window capture via `x11rb` (reading
  `_NET_ACTIVE_WINDOW` + `_NET_WM_NAME`), clipboard paste via `enigo`.
- **Linux Wayland**: global hotkey via the XDG Desktop Portal
  (`org.freedesktop.portal.GlobalShortcuts`) using `ashpd`. Paste falls
  back to clipboard-only mode — the text is placed on the clipboard and
  a new `ClipboardOnly` transcription state surfaces a "press Ctrl+V to
  paste" toast. A detached thread holds the selection alive for up to
  60 s via `arboard::SetExtLinux::wait_until`.
- **`get_display_server` Tauri command** and a new `platform.displayServer`
  reactive field in the frontend, so components can render
  Wayland-specific UX (hidden hotkey picker, compositor-aware hints).
- **`docs/assets/banner.png`** — Claude-Design brand banner wired into the
  README header.

### Changed

- **`HotkeyManager` split into backends** — `rdev_backend` (Win/mac/X11)
  and `wayland_backend` (Wayland), selected at runtime. The orchestrator
  and all its tests remain platform-agnostic.
- **`tauri.conf.json` no longer carries a global `resources: ["*.dll"]`
  glob** — per-platform configs `tauri.windows.conf.json` +
  `tauri.linux.conf.json` are auto-merged by the Tauri CLI and declare
  only the resources that actually exist on each OS. The CI
  `ci-placeholder.dll` hack is retired.
- **`scripts/download-whisper-cli.sh` now builds `whisper-cli` from source
  on Linux** — whisper.cpp v1.7.6 ships no Linux binary. The script
  clones the pinned tag, invokes cmake with `-DBUILD_SHARED_LIBS=OFF
  -DGGML_NATIVE=OFF` (static + generic-CPU), and produces a
  portable binary.
- **README documents the supported-compositor matrix** (GNOME 48+ ✅,
  KDE Plasma 6+ ✅, Hyprland ⚠️, Sway/wlroots ⚠️ clipboard-only).

### Fixed

- `TextInjector::inject` now returns `InjectOutcome` so callers can
  distinguish a successful paste from a clipboard-only fallback.
- `arboard` selection no longer evaporates when our function returns on
  Wayland (was silently dropping the selection on handle drop).

### Known limitations

- Wayland paste is clipboard-only in v0.2. Seamless paste via
  `ashpd::RemoteDesktop` + libei is planned for v0.3.
- No focused-window tracking on Wayland. Architectural — there is no
  portal for it and none is planned upstream.
- Sway / wlroots compositors run in clipboard-only mode until
  `xdg-desktop-portal-wlr` implements GlobalShortcuts (upstream
  issue #240).
- `.deb` and `.AppImage` are unsigned, same as Windows installers.

## [0.1.0] — 2026-04-23

First public release for Windows. Feature-complete for single-user dictation;
macOS build will follow.

### Added

- Push-to-talk dictation with configurable global hotkey. Default on Windows
  is `Ctrl + Win` (modifier-only chord); Win key events are intercepted at OS
  level to suppress the Start menu and Win+Ctrl+V sound output picker.
- Hands-free lock mode: quick tap-tap of the hotkey starts a locked recording
  that keeps running until the user taps once more. Escape cancels.
- Local transcription via bundled `whisper.cpp` (sidecar) with downloadable
  models (`ggml-tiny` through `ggml-large-v3`), all SHA-256 pinned.
- Groq Cloud engine with real-time model catalog merge (static + live
  `/v1/models`) and circuit breaker (5 failures in 60s → open for 30s).
- History viewer: local SQLite, segmented filter by engine, Ctrl+F to focus
  search, sticky day headers with word counts, source-app tracking
  (`GetForegroundWindow`) so each row shows which app the user was in.
- First-run wizard: 5 steps (welcome / permissions / engine / setup / test)
  with a live dictation test in the final step that uses the chosen runtime.
- Custom vocabulary textarea seeded into Whisper's decoder prompt.
- Real-time VU meter in Settings → Micrófono (cpal input stream at 30 Hz).
- Monthly Groq cost alert fired once per month via system notification.
- Boot-on-start via `tauri-plugin-autostart`.
- Alt+Shift+Z shortcut: re-paste the last successful transcription.
- Overlay pill with three states (recording / transcribing / error), animated
  via `border-color` morph + content crossfade + slide-up entrance.
- Failed-WAV preservation: transcription failures save the audio to
  `~/.quill/failed/` for 24 h, allowing retry from Historial.
- Tray icon that reflects current state (idle / recording / error).
- Custom Discord-style titlebar with integrated brand and window controls.

### Known limitations

- Windows only. macOS requires the Darwin whisper-cli binary and the
  Accessibility / Input Monitoring permission flow — both pending.
- Installer is unsigned; Windows SmartScreen will warn on first run.
- No auto-update mechanism. Upgrade by downloading the next installer.
- Single-user. Designed for one person's machine; no multi-profile support.
- Vocabulary affects Whisper's decoder as a prompt; there's no
  post-transcription exact-match substitution yet.

[Unreleased]: https://github.com/lautarosegura/quill/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/lautarosegura/quill/releases/tag/v0.2.1
[0.2.0]: https://github.com/lautarosegura/quill/releases/tag/v0.2.0
[0.1.0]: https://github.com/lautarosegura/quill/releases/tag/v0.1.0
