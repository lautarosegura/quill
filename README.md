<!-- Banner generated via Claude Design — replace the line below with:
     <p align="center"><img src="docs/assets/banner.png" alt="Quill" width="800" /></p> -->
<!-- banner placeholder — to be added before the v0.1.0 release is announced -->

<p align="center">
  <a href="https://github.com/lautarosegura/quill/releases/latest"><img alt="Latest release" src="https://img.shields.io/github/v/release/lautarosegura/quill?style=flat-square&color=5b3fd8" /></a>
  <img alt="Platform" src="https://img.shields.io/badge/platform-windows-5b3fd8?style=flat-square" />
  <img alt="License" src="https://img.shields.io/badge/license-MIT%20%7C%20Apache--2.0-5b3fd8?style=flat-square" />
  <a href="https://github.com/lautarosegura/quill/issues"><img alt="Issues" src="https://img.shields.io/github/issues/lautarosegura/quill?style=flat-square&color=5b3fd8" /></a>
</p>

# Quill

Voice-to-text dictation for Windows. Hold a hotkey, speak, release — the transcript appears wherever your cursor is. Runs [whisper.cpp](https://github.com/ggerganov/whisper.cpp) locally (private, offline) or [Groq Cloud](https://groq.com/) (fast, ~$0.10 / month typical). Built with [Tauri 2](https://tauri.app/) + [Svelte 5](https://svelte.dev/).

> **Status — v0.1.0, feature-complete for Windows.** macOS build is planned for v0.2 (waiting on the darwin whisper-cli binary + Accessibility permission flow).

## Install

1. Download **`Quill_0.1.0_x64-setup.exe`** from the [latest release](https://github.com/lautarosegura/quill/releases/latest).
2. Run the installer. Windows will show a **SmartScreen** warning because the binary is unsigned — click **More info → Run anyway**. (Signed releases are on the roadmap.)
3. On first launch, a 5-step wizard walks you through engine choice, model download, and a live dictation test.

That's it. The app launches minimized to the tray; hit your hotkey in any window to dictate.

## Features

- **Push-to-talk** with `Ctrl + Win` as the default (configurable). Win key events are intercepted at OS level so the Start menu doesn't open on release.
- **Hands-free lock mode** — tap-tap the hotkey quickly to start a recording that keeps going until you tap once more. Escape cancels.
- **Local or cloud engine** — Whisper on-device (private, offline, $0) or Groq Cloud (~500 ms latency). Switch per-session in Settings.
- **Historial** — every transcription saved to local SQLite, with search, engine filter, and source-app tracking (shows which window you were in when you pressed the hotkey).
- **Alt + Shift + Z** to re-paste the last transcription without recording again.
- **Live VU meter** in Settings so you can verify your mic before dictating.
- **Overlay pill** at a configurable corner shows recording / transcribing / error states.
- **Custom vocabulary** seeded into Whisper's decoder prompt so names, jargon, and acronyms transcribe correctly.
- **Monthly Groq cost alert** — set a threshold, get a system notification when you cross it.

## Where your data lives

All state is stored under `~/.quill/` on your machine:

| File | Purpose |
| ---- | ------- |
| `config.json` | Settings (engine, hotkey, language, etc.) |
| `history.db` | Transcription history (SQLite) |
| `models/` | Downloaded Whisper models |
| `vocabulary.txt` | Custom vocabulary prompt |
| `failed/` | Preserved audio from failed transcriptions (swept after 24 h) |
| `logs/` | App logs |
| `alert_state.json` | Monthly cost-alert "last fired" tracker |

Your **Groq API key is stored in the Windows Credential Manager**, never in `config.json`. No telemetry, no analytics — the only data that leaves your computer is the audio you explicitly send to Groq (when Groq is the active engine).

## Keyboard shortcuts

| Shortcut | Action |
| -------- | ------ |
| `Ctrl + Win` (default) | Push-to-talk — hold to record, release to transcribe |
| tap-tap `Ctrl + Win` quickly | Start locked hands-free recording (tap once more to finish) |
| `Escape` while locked | Cancel the in-flight recording |
| `Alt + Shift + Z` | Re-paste the last successful transcription |
| `Ctrl + F` / `Ctrl + K` in Historial | Focus the search bar |

## Screenshots

<!-- Fill in with exported Claude Design screens or live captures. -->

_Coming soon — grab some screens after the first release._

## Development

### Prerequisites

- [Rust 1.77+](https://rustup.rs/)
- [Node.js 20+](https://nodejs.org/) and [pnpm 9+](https://pnpm.io/)
- [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your platform

### Setup

```bash
git clone https://github.com/lautarosegura/quill
cd quill
pnpm install

# Fetch the whisper-cli sidecar binary for your platform
./scripts/download-whisper-cli.sh        # macOS / Linux
.\scripts\download-whisper-cli.ps1       # Windows PowerShell
```

### Daily commands

| Command | Purpose |
| ------- | ------- |
| `pnpm tauri dev` | Dev mode with hot reload |
| `pnpm check` | Svelte + TypeScript type-check |
| `pnpm build` | Frontend production build |
| `cargo test --manifest-path src-tauri/Cargo.toml --lib` | Backend unit + integration tests |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --lib -- -D warnings` | Backend lint |
| `pnpm tauri build` | Bundled installer (MSI + NSIS on Windows) |

Set `RUST_LOG=debug` before `pnpm tauri dev` to see verbose backend logs in the console.

### Project layout

```
quill/
├── src/                    ← Svelte frontend (main, overlay, wizard windows)
├── src-tauri/              ← Rust backend
│   ├── src/                  ← modules (config, orchestrator, hotkey, engines, …)
│   ├── binaries/             ← whisper-cli sidecar + DLLs (gitignored, populated by script)
│   ├── icons/                ← branded app + tray state icons
│   └── tools/                ← helpers (make_icons.ps1)
├── design/reference/       ← Claude Design bundle — source of truth for UI
├── docs/
│   ├── assets/               ← banner + screenshots
│   └── testing/              ← manual E2E checklist
└── scripts/                ← sidecar download + setup helpers
```

## Known limitations (v0.1.0)

- **Windows only** — macOS build pending
- **Unsigned installer** — SmartScreen warning on first launch
- **No auto-update** — upgrade by downloading the next installer
- **Single-user** — designed for one person's machine
- **Vocabulary as prompt** — seeds Whisper's decoder, no post-transcription exact-match substitution yet

See [CHANGELOG.md](CHANGELOG.md) for the full feature history.

## Contributing

Bug reports and feature requests welcome via [GitHub Issues](https://github.com/lautarosegura/quill/issues) — there are templates to make it quick. For open-ended questions, use [Discussions](https://github.com/lautarosegura/quill/discussions).

## License

Dual-licensed under MIT OR Apache-2.0, at your option. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).

## Credits

Built on the shoulders of:

- [whisper.cpp](https://github.com/ggerganov/whisper.cpp) — local speech-to-text
- [Groq](https://groq.com/) — fast cloud transcription API
- [Tauri](https://tauri.app/) + [Svelte](https://svelte.dev/) — desktop app runtime and UI framework
- [Lucide](https://lucide.dev/) — icon glyphs
