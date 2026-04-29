# Changelog

All notable changes to Quill are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and Quill uses
[semantic versioning](https://semver.org/). Pre-1.0 releases may include
breaking changes between minor versions.

## [Unreleased]

## [0.5.1] — 2026-04-29

Quick patch addressing a usability bug surfaced after v0.5.0 testing.

### Fixed

- **Silent captures no longer produce hallucinated transcriptions.** A
  quick accidental hotkey tap (or holding the key without speaking) was
  feeding Whisper near-empty audio, which would hallucinate filler text
  like "thanks for watching" or "you". Two layered fixes:
  - Default `min_duration_ms` raised from 250 → 400ms so accidental
    quick-taps are discarded before transcription. Existing installs
    keep their stored value.
  - New audio energy gate in the orchestrator: `AudioRecorder::stop_recording`
    now returns an `AudioCapture` with `peak`, `rms`, and `duration_ms`
    alongside the WAV bytes. Captures with `peak < 0.02` (~-34 dBFS)
    skip transcription entirely and the pill returns to Idle. Peak
    (not RMS) is the discriminator because short words like "OK" have
    a low RMS — most of their 300ms is silence between phonemes — but
    a high peak when the user vocalizes.

## [0.5.0] — 2026-04-28

The "LLM polish" pack — an opt-in cleanup stage that runs after Whisper
transcription and before injection. Removes muletillas (eh, mm, o sea),
normalizes punctuation, and produces text that reads like writing
instead of speaking — without touching meaning, vocabulary, or
rewriting style. Off by default. Pure addition over v0.4.0.

### Added

- **Three cloud LLM providers** — Groq, Anthropic, OpenAI. Each has its
  own keychain slot and its own preferred model. Switching providers is
  a config write, not an API call. Local LLM via llama.cpp sidecar is
  deferred to v0.6+ (would duplicate the whisper-cli pattern).
- **New Settings accordion: "Pulido con IA"** — third section, after
  "Motor de transcripción". Master toggle, segmented provider switcher
  with brand-tinted active dots + green "configured" pip per provider,
  per-provider API key panel with 4 states (empty / editing / saved /
  invalid), model dropdown with one-line blurbs, editable system prompt
  textarea (max 2000 chars, debounced auto-save, "Restaurar default"),
  and a live preview pane.
- **Live preview pane** with input → arrow → output layout. Renders an
  inline word-level diff with `.diff-add` / `.diff-rem` highlights
  (toggleable via "Resaltar cambios"), latency + token counts in
  monospace metadata, and dedicated error states (network /
  unauthorized / rate-limit) — surfaced in the preview, never
  in the dictation pipeline.
- **Pipeline integration** in `orchestrator.rs`: polish runs after
  vocabulary substitution, before injection. When `llm_polish_enabled
  = false` the call is a config-read no-op. On any failure (network,
  auth, rate-limit) the orchestrator logs a warning and uses the
  unpolished text — never punishes dictation for a flaky API.
- **Six new Tauri commands** — `get/set/delete/test_llm_polish_key`,
  `test_llm_polish` (preview), `list_llm_polish_models`. All
  per-provider where applicable.
- **21 unit tests** for the three providers (happy path, 401, 429,
  500-class errors, timeouts, connection refused) using mocked HTTP
  via `wiremock`. 4 dispatcher tests cover the toggle + provider
  switching logic. 133 total tests pass.
- **`IconSparkles`** added to the design system for the AI-touched
  feature affordance.

### Changed

- `Config` gains five `#[serde(default)]`-gated fields:
  `llm_polish_enabled`, `llm_polish_provider`, `llm_polish_models`,
  `llm_polish_system_prompt`, `llm_polish_max_input_chars`. Existing
  `~/.quill/config.json` files load unchanged.
- `SecretStore` generalized into per-provider `get/set/delete_llm_key`
  methods. The transcription Groq key (`groq_api_key`) is intentionally
  separate from the polish Groq key (`groq_llm_key`) so the two roles
  can be revoked independently.

## [0.4.0] — 2026-04-27

The "Linux compatibility round-out" — three additive changes that bring
Quill from "works on Linux if you're on the right config" to "works on
every Linux desktop, with installation as easy as the rest of the
ecosystem." Nothing here changes the experience on Windows, macOS, X11,
GNOME 48+, or KDE Plasma 6+ — it's pure additive coverage for everyone
else.

### Added

- **Flatpak / Flathub manifest** (`flatpak/com.lauta.quill.yml`) builds
  the entire stack — whisper.cpp v1.7.6, Silero VAD, libxdo, the
  libayatana-appindicator chain, and the Quill app itself — inside a
  Freedesktop Sdk 24.08 sandbox. Once submitted to Flathub, every Linux
  user can install via `flatpak install flathub com.lauta.quill`
  regardless of distro. Includes the AppStream metadata and XDG desktop
  entry required for software-center discovery.
- **Wizard adaptive setup card** — the permissions step now detects the
  user's Wayland compositor (GNOME version via `gnome-shell --version`,
  KDE Plasma version via `KDE_SESSION_VERSION`, or any Sway / Hyprland
  / wlroots variant via `XDG_CURRENT_DESKTOP`) and surfaces a copyable
  `sudo usermod -aG input $USER` instruction only on configs that lack
  the GlobalShortcuts portal. New `LinuxEnvironment` struct + Tauri
  command back the detection. Users on supported configs see no
  change; users on Sway / Hyprland / KDE Plasma 5 / GNOME ≤ 47 finally
  see *why* the hotkey fallback needs the extra step.
- **Hybrid feedback on Wayland** — for the two transcription states
  where the user has to *act* (`ClipboardOnly` and `Error`), Quill now
  also fires a native desktop notification via the
  `org.freedesktop.Notifications` D-Bus interface, in addition to the
  pill overlay. Solves the "where did the pill end up?" problem on
  compositor-managed Wayland sessions where `set_position` is
  silently ignored. X11 sessions and the regular record / transcribe /
  inject path are unchanged.

### Changed

- Multi-segment `XDG_CURRENT_DESKTOP` values (e.g. Ubuntu's
  `"ubuntu:GNOME"`) are now matched against every segment instead of
  only the first, fixing a pre-detection blind spot on Ubuntu and
  Pop!_OS where the wizard wouldn't have recognized GNOME at all.

## [0.3.0] — 2026-04-27

The "transcription quality pack" — three orthogonal improvements to how
Whisper transcribes the user's audio. All ship behind sensible defaults
so existing users feel an immediate quality bump without any setup.

### Added

- **VAD pre-processing** via Silero v6.2.0 (built into `whisper-cli`'s
  `--vad` flag). The VAD model (`ggml-silero-v6.2.0.bin`, ~865 KB) is
  bundled in the installer and passed alongside every transcribe call.
  Eliminates the trailing-silence hallucinations Whisper produces when
  the audio ends in silence ("you", "thanks for watching", repeated
  last phrase). On by default; `Config.vad_enabled` exposes the toggle
  for the rare power user who needs to disable.
- **Vocabulary post-substitution** — exact-match replacement applied
  after `post_process::clean`, with regex word-boundary semantics so a
  rule `Mokia → Nokia` doesn't touch "Mokian". Word boundaries are
  smart-elided when the pattern starts/ends with a non-word char (e.g.
  `"C++"` only gets a leading boundary). New "Sustituciones" table
  appended below the existing Vocabulario textarea — Reemplazar / Por
  / Aa case-sensitivity / delete columns plus "+ Agregar".
- **Custom prompt presets** — switchable Whisper-decoder prompts that
  bias transcription toward a context. 4 built-ins ship with the
  install: **General**, **Código**, **Email**, **Casual**. Users can
  create custom presets, edit prompts, delete (built-ins are
  edit-prompt-only, name-locked, non-deletable). Active preset's
  prompt is concatenated with `Config.vocabulary` and truncated to
  ~880 chars before being sent to Whisper — preset = tone, vocabulary
  = words, both bias the decoder.
- **Tray menu "Preset" submenu** — quick-switch the active preset
  without opening the app. Click any preset name to set it as active.
  Built once at app startup; user-created presets added at runtime
  appear after restart (dynamic rebuilding is queued for v0.3.x).
- **New `/main/presets` page** — polished master/detail UI for preset
  management. Master list (280 px) on the left with custom radio-button
  activation, "Sin preset" pseudo-row, and a divider between built-ins
  and custom presets; editor on the right with name + prompt + active
  toggle + delete. Custom radio (`.pradio`) supports independent select
  vs activate semantics — clicking a row body selects it for editing,
  clicking the radio activates it.
- **Polished Sustituciones table** on the Vocabulario page — grid-based
  inline-editable rows, "Aa" pill toggle for case sensitivity in
  explicit on/off variants, hover-revealed row delete with red-tinted
  hover, accent-CTA empty state. Generated from the Claude Design
  v0.3.0 bundle.
- **`IconPresets` SVG** — stack-of-cards 16×16 Lucide-style sidebar
  glyph for the new Presets nav entry. Replaces the Vocabulario icon
  used as a placeholder in earlier RCs.
- **`docs/claude-design-brief-v0.3.0.md`** — handoff document listing
  every new UI surface in v0.3.0 with functional spec + current Svelte
  source for context. Drove the Claude Design redesign of the
  substitutions table, presets master/detail, and the sidebar icon.

### Changed

- `regex = 1.11` added as a top-level dependency for the substitution
  word-boundary replacements.
- Orchestrator's prompt construction now goes through
  `Config::active_prompt()` — single source of truth that combines the
  active preset with the vocabulary and handles the 880-char truncation.

### Coexistence with `vocabulary`

The original v0.3.0 spec had presets *replacing* the vocabulary field.
We changed direction mid-implementation to **coexistence** — preset
gives the decoder the tone/style context, vocabulary gives it the
words list, both apply simultaneously. More expressive, no migration
needed. Setting `Config.active_preset_id = None` (the default for
existing configs) preserves pre-presets behavior exactly: vocabulary
becomes the entire prompt.

### Known limitations

- Tray submenu doesn't auto-refresh when the user creates a new preset
  via the Settings page; restart picks it up.
- No UI for VAD toggle; edit `~/.quill/config.json` directly to disable.

## [0.2.2] — 2026-04-24

Polish + Wayland round-out from v0.2.1 user testing.

### Fixed

- **Pill labels in Spanish** — the floating overlay said "Recording" /
  "Transcribing…" in English while the rest of the app was in Spanish.
  Now consistently "Grabando" / "Transcribiendo…". Affects all
  platforms (Windows, macOS, Linux), not just Linux.
- **Overlay invisible on Wayland multi-monitor** — `set_position` is
  silently ignored by GNOME for regular toplevel windows, and on
  multi-monitor setups our calculated coordinates can land on a
  monitor the user isn't focused on. Skip the call entirely on
  Wayland; let the compositor place the overlay.

### Added

- **State-aware tray icon tooltip** — the tray icon's hover tooltip
  now mirrors the transcription state (`Quill — Grabando`,
  `Quill — Transcribiendo`, `Quill — Listo para pegar (Ctrl+V)`,
  etc.). Reliable feedback across all compositors, complements the
  pill (and replaces it as the primary signal on Wayland multi-monitor
  setups where the pill placement is unpredictable).
- **Persistent Wayland RemoteDesktop restore token** — the consent
  dialog for libei keystroke synthesis now appears only on the very
  first paste of the install. Previously it re-popped on every app
  launch because the token lived in a process-wide static.
  `Config.wayland_remotedesktop_token` is now persisted to
  `~/.quill/config.json` and rehydrated on first paste of each launch.
- **`ROADMAP.md`** — captures the prioritized list of features and
  improvements considered but not scheduled, organized into Wayland
  stabilization / macOS support / UX features / ops + distribution /
  wild-card ideas.

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

[Unreleased]: https://github.com/lautarosegura/quill/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/lautarosegura/quill/releases/tag/v0.3.0
[0.2.2]: https://github.com/lautarosegura/quill/releases/tag/v0.2.2
[0.2.1]: https://github.com/lautarosegura/quill/releases/tag/v0.2.1
[0.2.0]: https://github.com/lautarosegura/quill/releases/tag/v0.2.0
[0.1.0]: https://github.com/lautarosegura/quill/releases/tag/v0.1.0
