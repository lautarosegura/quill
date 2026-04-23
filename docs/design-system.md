# Quill Design System

Source of truth: `design/reference/project/` (HTML + JSX delivered from Claude Design, 2026-04-22).

## Tokens

All tokens live as CSS custom properties in `src/app.css`. Components must reference tokens — never hard-code colors, sizes, or radii. Exception: hues for semantic states (success/warning/error) are inlined as OKLCH values where they appear (see "Semantic colors" below).

### Colors (OKLCH)

| Token | Dark | Light | Use |
|-------|------|-------|-----|
| `--accent` | `oklch(0.62 0.18 280)` | same | Primary interactive color (purple) |
| `--accent-soft` | `oklch(0.62 0.18 280 / 0.12)` | same | Accent background tint |
| `--accent-ring` | `oklch(0.62 0.18 280 / 0.35)` | same | Focus ring |
| `--bg` | `oklch(0.18 0.008 260)` | `oklch(0.985 0.003 80)` | App background |
| `--bg-elev` | `oklch(0.21 0.009 260)` | `oklch(1 0 0)` | Elevated surface (cards, buttons) |
| `--bg-panel` | `oklch(0.205 0.008 260)` | `oklch(0.975 0.003 80)` | Sidebar / secondary panels |
| `--border` | `oklch(1 0 0 / 0.07)` | `oklch(0 0 0 / 0.07)` | Hairline borders |
| `--border-strong` | `oklch(1 0 0 / 0.11)` | `oklch(0 0 0 / 0.12)` | Stronger dividers |
| `--text` | `oklch(0.96 0.005 260)` | `oklch(0.21 0.01 260)` | Primary text |
| `--text-dim` | `oklch(0.72 0.01 260)` | `oklch(0.42 0.01 260)` | Secondary text |
| `--text-mute` | `oklch(0.55 0.01 260)` | `oklch(0.58 0.01 260)` | Muted / placeholder |
| `--hover` | `oklch(1 0 0 / 0.035)` | `oklch(0 0 0 / 0.035)` | Row hover |
| `--active` | `oklch(1 0 0 / 0.06)` | `oklch(0 0 0 / 0.055)` | Active/selected row |

### Semantic colors (inlined, not tokens)

Used in toasts, banners, status indicators. If a third component needs one of these, promote to a token.

- **Success (green)**: `oklch(0.75 0.16 155)` · bg `oklch(0.72 0.16 155 / 0.1)` · border `oklch(0.72 0.16 155 / 0.25)`
- **Warning (amber)**: `oklch(0.78 0.14 75)` · bg `oklch(0.72 0.18 25 / 0.08)`
- **Error (red)**: `oklch(0.72 0.18 25)` · bg `oklch(0.65 0.18 25 / 0.1)` · border `oklch(0.65 0.18 25 / 0.3)` · solid fill `oklch(0.6 0.18 25)`
- **Recording pulse**: `#EF4444` (also used in Pill border)
- **Transcribing amber**: `#F59E0B`
- **Error solid**: `#F43F5E`

### Typography

- **Sans**: Inter · weights 400 / 500 / 600 / 700. Feature flags `cv02`, `cv03`, `cv04`, `cv11` enabled.
- **Mono**: JetBrains Mono · weights 400 / 500. Used for keybindings, version numbers, stats.

### Utility classes

Declared in `src/app.css`. Safe to use anywhere:

| Class | Effect |
|-------|--------|
| `.bg-app` · `.bg-elev` · `.bg-panel` | Surface backgrounds |
| `.text-base-strong` · `.text-base-dim` · `.text-base-mute` | Text hierarchies |
| `.border-hair` · `.border-hair-strong` | Border hierarchy |
| `.bg-hover` (applied on hover) · `.bg-active` | Interactive surfaces |
| `.accent-text` · `.accent-bg` · `.accent-soft` | Brand color variations |
| `.ring-accent` | Focus ring |
| `.row-focus` | Selected row style (Historial list) |
| `.fade-in` | 180ms entry animation |
| `.shimmer-text` | Shimmer on transcribing state |
| `.rec-dot` · `.rec-ring` | Recording pulse animations |

### Keyboard caps

- `kbd` or `.kbd-inline` — small, ~10.5px. Inline within text.
- `.kbd-big` — medium, ~12px. Standalone key display.

## Components

All primitives live under `src/lib/components/ui/`. See `src/routes/design/+page.svelte` for a running showcase with every variant side by side.

### Rule of thumb

Adding or editing UI? First check if the visual already exists in the design reference:

1. Look in `design/reference/project/src/` (JSX) — the design is organized by screen.
2. If the pattern is reused in 2+ screens, it's a primitive. Port to `$lib/components/ui/`.
3. If it's screen-specific, compose it inline from existing primitives.

### Never

- Hard-code colors, spacings, or radii. Use tokens.
- Re-implement a primitive instead of importing it.
- Skip verifying `/design` after UI changes.
