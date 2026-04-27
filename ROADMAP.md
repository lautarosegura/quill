# Roadmap

This document tracks features and improvements that have been considered but
not yet scheduled. Use it as a reference when planning future releases —
ordering and version assignments are intentionally absent so the document
ages well.

For shipped work, see [CHANGELOG.md](CHANGELOG.md).

## A. Wayland stabilization

Round-out of the v0.2 Linux push. Mostly small, mostly Linux-only.

- [x] **Persist `wayland_remotedesktop_token` in Config** — eliminates the
      RemoteDesktop consent dialog on every app launch (currently it
      re-prompts the first time per session). Shipped in v0.2.2.
- [x] **Skip `set_position` on Wayland** — multi-monitor compositors silently
      ignore the call and the overlay can land off-screen. Let the compositor
      place it instead. Shipped in v0.2.2.
- [ ] **Overlay redesign for compositor-agnostic feedback** — move from a
      custom transparent toplevel window (which Wayland regularly mishandles)
      to either layer-shell (wlroots-only) or a libnotify-based notification.
      The tray tooltip is the current reliable fallback but it requires
      hover; a passive visual would be better.
- [ ] **Wizard step that detects GNOME version** — show an informational
      card on first run if GNOME < 48 explaining that the user needs to be
      in the `input` group for the hotkey to work.
- [ ] **Third hotkey fallback via `tauri-plugin-global-shortcut`** — for
      compositors without the GlobalShortcuts portal AND users not in the
      `input` group. The plugin doesn't support press/release semantics so
      this is degraded UX (no hold-to-record), but it's zero-setup. Probably
      not worth the complexity unless we hit real users with that profile.

## B. macOS support

The big platform gap. Tauri supports macOS out of the box, but the surface
work is non-trivial.

- [ ] **`whisper-cli` build for Darwin** — same source-build approach as
      Linux. Apple Silicon + Intel both, statically linked.
- [ ] **`focused_window` macOS implementation** — `objc2_app_kit::NSWorkspace::frontmostApplication`
      then `localizedName`. Few lines of objc2 calls.
- [ ] **Permission flow** — Accessibility (for keystroke synthesis),
      Input Monitoring (for global hotkey via rdev), Microphone. Each has
      its own native prompt, its own `NSPrivacyAccessedAPITypes` manifest
      entry, its own entitlement, and its own user denial path. Wizard
      needs a dedicated step walking the user through all three.
- [ ] **Code signing + notarization** — Apple Developer account ($99/year),
      `codesign` step in CI, `notarytool submit`, staple. Without this the
      app shows the Gatekeeper "damaged or unverified" warning.
- [ ] **macOS-specific default hotkey** — Cmd-based, e.g. `Cmd+Shift+Space`.
      Currently the non-Windows default is `Ctrl+Shift+Space` which collides
      with macOS Spotlight in some configurations.
- [ ] **DMG installer testing** — Tauri builds DMGs but the icon, layout,
      and "drag to Applications" affordance need polish.

## C. UX features

Platform-agnostic improvements that deepen the dictation experience.

- [ ] **VAD (voice activity detection)** — auto-stop recording after N
      seconds of silence. whisper.cpp v1.7.6 ships built-in VAD; we just
      pass `--vad` and configure thresholds. Eliminates the need for the
      double-tap-lock workflow in many cases.
- [ ] **Translation mode** — `whisper-cli --translate` runs the model with
      a translate-to-English target. Adds a "Transcribe and translate"
      toggle in Settings or a dedicated hotkey.
- [ ] **Multi-language beyond ES/EN** — Whisper supports 99 languages.
      Pull the full list, expose a dropdown, persist last-used. Auto-detect
      mode is also possible (`--language auto`).
- [ ] **Vocabulary post-substitution** — current vocabulary is fed to
      Whisper as a prompt (decoder bias). A second pass after transcription
      could do exact-match replacements for terms Whisper consistently
      hallucinates wrong (e.g. brand names, internal jargon).
- [ ] **Hotkey to toggle engine** — separate global shortcut to flip
      Local ↔ Groq without opening Settings. Useful when on bad wifi.
- [ ] **Custom prompt presets** — "email mode", "code mode", "casual mode"
      with different system prompts and post-processing rules. Switch via
      tray menu or hotkey.
- [ ] **Per-app prompt context** — detect focused app, prepend a tailored
      prompt (e.g. "user is in VSCode, may dictate code-style identifiers").
      Builds on the existing `source_app` capture.
- [ ] **Language toggle hotkey** — `language_cycle_hotkey` field already
      exists in Config but is currently unused. Wire it up.
- [ ] **Better error UX** — current Pill says "Error" with a message; could
      offer a one-click "Reintentar" right in the overlay for failed
      transcriptions, matching the existing Historial retry flow.

## D. Operations / distribution

Important once the audience grows beyond personal use.

- [ ] **Auto-update via `tauri-plugin-updater`** — checks GitHub Releases
      for newer versions, prompts the user, downloads, replaces. Needs
      signature verification (Tauri uses Ed25519).
- [ ] **Signed Windows installer** — Authenticode certificate (~$300/yr
      from DigiCert / Sectigo). Eliminates SmartScreen warning. Hardware
      token (FIPS-compliant) required as of June 2023.
- [ ] **Flatpak / Flathub** — the standard modern Linux distribution
      channel. Sandboxed runtime, automatic updates, works on every distro.
      Manifest in `org.flathub.Quill.json`. Some portal interactions need
      to be wired through the Flatpak portal proxy.
- [ ] **Snap (Canonical store)** — Ubuntu-first distribution. Different
      sandbox, different packaging. Lower priority than Flatpak (smaller
      reach outside Ubuntu).
- [ ] **Microsoft Store** — MSIX-packaged version of the Windows build.
      Auto-update built-in, no SmartScreen issues. Tauri can build MSIX.
- [ ] **Marketing site** — landing page with screenshots, demo video,
      feature list, install links. Probably hosted on GitHub Pages.
- [ ] **Opt-in telemetry** — anonymous error reports + feature usage
      counts. Helps prioritize. Must be off-by-default and clearly
      explained.

## E. Wild card features

Bigger bets that could open new use cases.

- [ ] **Headless / scriptable mode** — Quill exposes an HTTP API on
      `localhost:33333` (or similar). External programs can POST audio /
      receive text, integrate with vim, scripts, other editors. The
      transcription engine stack already exists; just needs an HTTP layer.
- [ ] **Voice commands** — phrases like "delete last word", "new
      paragraph", "punto y aparte" trigger post-transcription edits
      instead of being literal text. Configurable per language. Builds
      on the post-process module.
- [ ] **History export** — JSON / TXT / CSV dump of the local SQLite
      history. Useful for backup, analytics, training data for finetuning.
- [ ] **Fine-tuned local model** — train Whisper LoRA on the user's own
      audio history, improving recognition of their specific voice
      and vocabulary. Big undertaking, may not justify cost vs cloud APIs.
- [ ] **Streaming transcription** — show partial results as the user
      speaks, matching Otter / Aiko UX. whisper.cpp's `stream` example is
      a starting point; would require a different UI pattern (no overlay
      pill, more like a sticky transcription window).
- [ ] **Multi-device clipboard sync** — Quill on laptop + desktop both
      see the last transcription. Risky (cloud roundtrip, privacy
      tradeoffs). Probably out of scope for the privacy-first positioning.

## How this list gets updated

When a feature ships, move its checkbox to checked here and add the
proper entry in `CHANGELOG.md` under the release. Drop the checkbox
when the feature is finalized in the changelog. When a new idea comes
up, append it under the most relevant section without worrying about
ordering — section ordering is rough priority but item ordering inside
each section isn't.
