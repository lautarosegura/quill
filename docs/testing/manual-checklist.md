# Quill — Manual End-to-End Testing Checklist

Last run: YYYY-MM-DD
Tester:
Build / commit:
OS:

Tick every box you actually exercised. If an item fails, file it in the "Found bugs" section at the bottom with severity (blocker / high / medium / low) and the scenario name.

---

## 1. Pre-flight (build verification)

- [ ] `cargo test --manifest-path src-tauri/Cargo.toml` exits 0, no ignored failures
- [ ] `pnpm check` passes with zero Svelte/TS errors
- [ ] `pnpm build` completes, produces a `src-tauri/target/release` artifact
- [ ] `pnpm tauri dev` launches without a red error panel
- [ ] `~/.quill/` directory is created on first launch if it did not exist
- [ ] No panics in the Rust console during idle (leave running 60 s)
- [ ] No unhandled promise rejections in the DevTools console during idle

## 2. First-run wizard

Reset state before each sub-scenario: rename `~/.quill/config.json` to `config.json.bak`, restart the app.

### 2a. Wizard — Local engine only
- [ ] Wizard auto-opens (WizardWindow 640x500) when config.json is missing
- [ ] Step 1 (welcome) renders, "Siguiente" advances
- [ ] Step 2 (engine choice) — pick "Local"; "Siguiente" enabled
- [ ] Step 3 (model pick) — default model (turbo) preselected, download starts, progress bar advances to 100%
- [ ] Step 4 (hotkey) — default `Ctrl+Shift+Space` shown, press a new combo, it captures correctly
- [ ] Step 5 (done) — "Finalizar" closes wizard and opens MainWindow
- [ ] `~/.quill/config.json` now exists, contains `engine: "local"` and the chosen hotkey
- [ ] `~/.quill/models/ggml-*.bin` exists for the chosen model

### 2b. Wizard — Groq cloud only
- [ ] Repeat with engine = "Groq", paste API key, Test button returns "API key válida"
- [ ] Wizard skips local model download step
- [ ] config.json has `engine: "groq"`, no local model field required
- [ ] Groq API key persisted in OS keychain (not in plaintext config.json)

### 2c. Wizard — Both engines
- [ ] Select "Both", wizard walks through model download AND Groq key steps
- [ ] config.json reflects both, preferred engine matches the radio picked last

## 3. Happy-path dictation — Local engine

Set engine = local, model = turbo, hotkey = **Ctrl+Win** on Windows / Ctrl+Shift+Space on macOS.

- [ ] Focus Notepad, hold hotkey, speak "hola mundo", release — "hola mundo" appears at cursor
- [ ] Focus VS Code editor, dictate short phrase — text inserted at caret, no extra newline
- [ ] Focus Slack message box — text inserted, Slack does not auto-send
- [ ] Focus Chrome omnibar — text appears in URL bar, not the page
- [ ] Focus Discord text channel — text inserted, cursor remains in field
- [ ] Dictation with accented chars ("canción, niño, ñandú") — diacritics preserved
- [ ] Historial shows new entry immediately after release, badge = "Local"

## 4. Happy-path dictation — Groq cloud engine

Swap engine to Groq in Ajustes.

- [ ] Hold hotkey, dictate same phrase — text inserted, Historial badge = "Groq"
- [ ] Latency feels under ~2 s for a 5-second utterance (note the time)
- [ ] Consecutive dictations (5 in a row) all succeed, no stuck overlay
- [ ] Works with VS Code and Notepad as smoke target apps

## 5. Engine swap at runtime

- [ ] In Ajustes, change engine from Local to Groq — no restart prompt
- [ ] Next dictation uses Groq (verify badge in Historial)
- [ ] Swap back to Local — next dictation uses local
- [ ] Change local model from turbo to large-v3 in Modelos, download completes
- [ ] Next dictation after swap uses large-v3 (check logs or Historial metadata)
- [ ] Invalid model selected and file missing — clear error, app does not crash

## 6. Hotkey hot-swap

- [ ] In Ajustes, change hotkey to `Ctrl+Alt+D`
- [ ] Old hotkey (Ctrl+Win on Windows, Ctrl+Shift+Space on macOS) no longer triggers recording
- [ ] New hotkey starts recording; release stops it, transcription inserted
- [ ] No app restart was needed
- [ ] Hotkey conflict with system shortcut is detected and surfaced (try `Ctrl+C`)
- [ ] Reset-to-default button restores the platform default

### 6a. Ctrl+Win default (Windows only) — Start menu suppression
- [ ] Default hotkey on a fresh install is Ctrl+Win (modifier-only chord, no trigger key)
- [ ] Hold Ctrl+Win and release — **the Start menu does NOT open**. This is the critical fix: `rdev::grab` consumes the Win-up event for chords that include Meta
- [ ] Hold Ctrl+Win, dictate, release — text inserts, no Start menu
- [ ] Hold only Win (no Ctrl), tap and release — Start menu opens normally (we don't suppress unrelated Meta events)
- [ ] Hold Ctrl+Win + arrow keys — does NOT trigger virtual desktop switching (our chord is already active by the time the arrow press arrives; but confirm by releasing all first)
- [ ] Switch to a custom hotkey (Ctrl+Alt+D), then switch back to default: recovers without restart
- [ ] In Settings, the KeyCapture widget shows "Ctrl + Win/⌘" without a trailing "+" and without a third KeyCap
- [ ] Capturing a new modifier-only chord: hold Ctrl+Alt, release both → saved as `{modifiers: ["ctrl","alt"], key: null}`

## 7. Historial

Seed with at least 6 dictations across 2+ days (fake the 2nd day by shifting clock or use existing entries).

- [ ] All dictations listed in reverse chronological order
- [ ] Badge (Local / Groq) matches the engine used per row
- [ ] Entries grouped by day header (Hoy, Ayer, explicit date)
- [ ] Search bar — type substring, list filters live
- [ ] Search is case insensitive ("HOLA" matches "hola mundo")
- [ ] Search across accented chars ("cancion" matches "canción") — note behavior
- [ ] Copy action — click copy on a row, paste into Notepad, content matches exactly
- [ ] Reinject action — click, text re-inserts at current cursor in focused app
- [ ] Delete single row — confirm modal, row disappears, DB row gone
- [ ] "Borrar todo" — confirm modal appears, accepting clears entire list + DB table
- [ ] Cancel on "Borrar todo" leaves Historial untouched
- [ ] Empty state renders when list is empty (no crash, helpful copy)

## 8. Modelos

- [ ] Hardware banner shows CPU, RAM, GPU (if any), matches Task Manager numbers roughly
- [ ] Local model list shows tiny / base / small / medium / large-v3 / turbo with sizes
- [ ] Download new model — progress bar animates 0 to 100, row shows "Instalado"
- [ ] Cancel mid-download — partial file cleaned up, no ghost row
- [ ] Delete local model — confirm modal, file removed from `~/.quill/models/`
- [ ] Active model cannot be deleted (or deletion auto-switches to fallback)
- [ ] Groq cloud section fetches and lists live models from Groq API
- [ ] Swap Groq model — next dictation uses the new one (verify via logs)
- [ ] Offline: Groq model list shows an error/empty state rather than spinning forever

## 9. Uso

- [ ] Top 3 stat cards (total dictations, total minutes, total cost) match Historial reality
- [ ] 30-day BarChart renders with bars on days you dictated, empty days flat
- [ ] Hover/tooltip on a bar shows the day's count
- [ ] Engine split (donut or bar) — Local % + Groq % = 100 exactly
- [ ] Cost alert input accepts decimal (e.g. `5.00`), persists after restart
- [ ] Set alert below current cost — UI shows red warning state
- [ ] Set alert above current cost — UI shows normal state
- [ ] Invalid input (letters, negative) rejected or sanitised

## 10. Vocabulario

- [ ] Paste `Quill, Tauri, Svelte` into vocab box, save
- [ ] Baseline dictation of "Quill" before adding vocab — record what came out
- [ ] Dictation of "Quill is built on Tauri and Svelte" after — "Quill", "Tauri", "Svelte" spelled exactly
- [ ] Remove vocab, dictate same phrase — may revert to phonetic spelling
- [ ] Very long vocab list (50+ terms) — still saves, does not lag input
- [ ] Special chars in vocab (`ñ`, hyphens, apostrophes) saved round-trip

## 11. Error paths

- [ ] Disconnect internet, engine = Groq, attempt dictation — overlay goes to error state, toast "sin conexión" or similar, no silent drop
- [ ] Set Groq API key to garbage, click Test — "API key inválida" message
- [ ] Save invalid key anyway, attempt dictation — friendly error, app survives
- [ ] Deny mic permission (Windows Settings → Privacy → Microphone = off for Quill), attempt dictation — clear "no mic access" error
- [ ] Grant mic again, next dictation works without restart
- [ ] Manually delete active local model file, attempt dictation — error "model not found", link or CTA to re-download
- [ ] Corrupt config.json (invalid JSON) — app falls back to defaults or shows recoverable error, does not crash loop
- [ ] Launch with history.db locked by another process — graceful error, not a hard crash

## 12. Edge cases

- [ ] Very short tap (<250 ms) — no overlay, no history entry, no insertion
- [ ] Very long dictation (60+ s) — cut at `max_duration_secs`, partial transcription still inserted
- [ ] Total silence for 10 s then release — empty transcription, no phantom insertion, history entry either omitted or marked empty
- [ ] Noisy background (music, fan) — transcription still plausible, no crash
- [ ] Hold hotkey, switch focus mid-dictation to another app, release — text inserted into the final focused app (document expected behavior)
- [ ] Press hotkey twice rapidly — second press ignored while first is active, no double-start
- [ ] Lock screen mid-dictation — recording cancels cleanly on resume

## 13. Close-to-tray

- [ ] Click MainWindow X — window hides, process still running (Task Manager)
- [ ] Tray icon visible bottom-right (Windows)
- [ ] Left-click tray icon — MainWindow reopens at last position/size
- [ ] Right-click tray icon — menu shows (Abrir, Dictado, Salir, etc.)
- [ ] "Salir" from tray menu — process exits, icon disappears
- [ ] Hotkey still works while MainWindow is hidden via tray
- [ ] Double tray icons never appear on repeated hide/show cycles

## 14. Overlay pill

- [ ] Press hotkey — pill appears bottom-center at 220x48
- [ ] Recording state visually red (or configured accent)
- [ ] Release hotkey — pill transitions to yellow/transcribing state
- [ ] On success — pill fades out
- [ ] On error — pill shows red-with-X, lingers a few seconds then dismisses
- [ ] Overlay does not steal focus — target app caret stays active, typing still works if you interrupt
- [ ] Overlay does not appear in Alt+Tab switcher
- [ ] Overlay does not appear on taskbar
- [ ] Overlay is click-through / always-on-top as expected
- [ ] Moving a fullscreen app (game, video) — overlay still renders on top during dictation

## 15. Settings persistence

Change each value, fully quit + relaunch, verify each persisted.

- [ ] Hotkey combo persists
- [ ] Language (ES / EN) persists — UI strings swap correctly
- [ ] Overlay position (bottom / top / custom) persists
- [ ] Max duration slider value persists
- [ ] Auto-launch-on-boot toggle persists (check via Task Manager startup tab)
- [ ] Telemetry / privacy toggles persist
- [ ] Theme choice persists
- [ ] Preferred engine persists
- [ ] Groq API key persists across restart (still in keychain, not exposed in config.json)

## 16. Theme toggle

- [ ] Sun/moon icon in sidebar toggles theme immediately
- [ ] Historial view readable in both themes
- [ ] Ajustes readable in both themes
- [ ] Vocabulario readable in both themes
- [ ] Modelos readable in both themes (progress bar visible in both)
- [ ] Uso charts readable in both themes (bar fills use theme tokens, not hardcoded)
- [ ] Wizard windows reflect theme if reopened
- [ ] Overlay pill respects theme

## 17. Design system regression (dev mode only)

- [ ] Open DevTools, run `location.href='/design'`
- [ ] Page loads, no red console errors
- [ ] Buttons (primary, secondary, ghost, danger) render with correct styles
- [ ] Inputs, selects, toggles, sliders render and interact
- [ ] Modals open and close without layout shift
- [ ] Toasts fire and auto-dismiss
- [ ] Cards, badges, tables render
- [ ] Progress bar primitive animates
- [ ] Icons load (no broken image placeholders)

## 18. Platform differences (Windows-specific)

- [ ] Hotkey uses Win32 registration, no conflict with OS hotkeys documented
- [ ] Tray icon renders with correct 16x16/32x32 DPI on 100% and 150% scaling
- [ ] Window chrome matches Windows 11 style (rounded corners, native min/max/close)
- [ ] SmartScreen warning on first launch of unsigned build — document whether expected
- [ ] `~/.quill/` resolves to `C:\Users\<user>\.quill\` (not AppData)
- [ ] Text insertion works in UWP apps (e.g. Terminal, Notepad store version) and Win32 apps

## 19. macOS to verify separately

(Tester on Windows — these must be run on a Mac before release.)

- [ ] First-run Accessibility + Microphone + Input Monitoring permission prompts appear
- [ ] Traffic light window decorations render correctly on MainWindow and WizardWindow
- [ ] Tray item appears in top-right menu bar, not dock
- [ ] Hotkey registration survives macOS fast-user-switching
- [ ] `~/.quill/` resolves to user home, config writes succeed under sandboxing rules
- [ ] Text insertion works in native Cocoa apps, Electron apps (Slack, Discord), and web browsers

---

## 20. Phase 6 polish — new behaviors

### 20a. Failed WAV preservation + retry
- [ ] Trigger a transcription failure (unplug mic mid-dictation, or invalidate Groq key, or delete the local model file). Entry appears in Historial marked "Falló" with an error reason
- [ ] The failed entry shows a **Reintentar** button on hover (only for engine-transcription failures, not inject failures)
- [ ] Click **Reintentar**: the captured WAV is re-transcribed with the current config, text inserts into the focused app, and a new success entry appears at the top of Historial
- [ ] The original failed entry remains in Historial (for audit); the WAV file at `~/.quill/failed/*.wav` is deleted after a successful retry
- [ ] Leave a failed entry un-retried for >24h: on next app startup, its WAV file is swept; the Historial row still exists but the **Reintentar** button is gone (no file to retry)
- [ ] `~/.quill/failed/` directory is auto-created on first app launch

### 20b. Cost alert notification
- [ ] Settings → **Alerta de gasto mensual (Groq)**: toggle OFF persists as `monthly_cost_alert_usd: null` in `~/.quill/config.json`
- [ ] Toggle ON with a threshold (e.g. $0.01): value persists; number input is inline and debounced
- [ ] Do Groq dictations until month cost crosses the threshold: a native system notification fires ("Alerta de gasto — Quill / Ya gastaste $X USD en Groq este mes")
- [ ] Notification fires exactly once per month — subsequent Groq dictations in the same month do NOT re-alert
- [ ] `~/.quill/alert_state.json` appears only after the first alert fires; contains `{"last_fired_month":"YYYY-MM"}`
- [ ] Turning the toggle OFF and back ON preserves the threshold value

### 20c. Boot-on-start
- [ ] Settings → **General** → toggle "Iniciar Quill al arrancar el sistema": turning ON registers an OS autostart entry (Windows: HKCU\...\Run\quill; macOS: ~/Library/LaunchAgents/; Linux: ~/.config/autostart/)
- [ ] Toggle OFF removes the registry/LaunchAgent entry cleanly
- [ ] After reboot with the toggle ON, Quill launches automatically (may be minimized to tray — verify tray presence)
- [ ] Auto-launched instance starts with `--autostart` arg; does NOT auto-show the main window

### 20d. Sound beeps on record start/end
- [ ] Settings → **General** → toggle "Sonidos al empezar y terminar dictado" OFF: no audio plays during dictation
- [ ] Toggle ON: a short tone plays when recording starts, another when it ends (transitions out of Recording state)
- [ ] Beeps don't play for short-tap ignored (held < min_duration_ms) — verify by a quick tap
- [ ] Volume is subtle, not jarring — should not drown out the dictation microphone capture or be louder than a Slack notification
- [ ] If audio output is unavailable (no default device), app doesn't crash — a log line reports beeps disabled

### 20e. Tray icon state-based colors
- [ ] Tray icon is gray / muted when Idle
- [ ] Tray icon turns red (or brand recording color) while Recording and through Transcribing
- [ ] Tray icon turns amber on Error; reverts to gray on the next Idle
- [ ] Swap is live — visible within 100 ms of state change

### 20f. Custom Quill app icon
- [ ] Taskbar / Dock icon is the purple quill glyph (no longer the Tauri default)
- [ ] Alt-Tab preview shows the branded icon
- [ ] File Explorer → installed `.exe` displays the custom icon
- [ ] Windows Start Menu tile (after install): shows the Quill icon at all sizes
- [ ] Installer .msi / .nsis carries the branded icon

---

## Found bugs

Format: `- **Severity · scenario**: description`

- **Severity · scenario**: example placeholder, delete when filing real ones
