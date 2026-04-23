# Changelog

All notable changes to Quill are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and Quill uses
[semantic versioning](https://semver.org/). Pre-1.0 releases may include
breaking changes between minor versions.

## [Unreleased]

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

[Unreleased]: https://github.com/lautarosegura/quill/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/lautarosegura/quill/releases/tag/v0.1.0
