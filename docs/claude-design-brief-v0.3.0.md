# Claude Design brief — Quill v0.3.0 transcription quality components

## What this is

A handoff brief in the **opposite direction** of the original Quill bundle.
The original `design/reference/project/` was Claude Design → coding agent.
This brief is coding agent → Claude Design: 3 new components were added
in v0.3.0 with deliberately-minimal styling, and we need them redesigned
to match the existing aesthetic before the release.

## Project context

Quill is a voice-to-text dictation app for Windows / Linux / macOS, built
with Tauri 2 + Svelte 5. The visual system is already established in the
8 existing screens — see `design/reference/project/Quill - App Screens.html`
for the full HTML/CSS source of every page (Historial, Settings,
Vocabulario, Modelos, Uso, Wizard, Overlay, Main shell). The new
components must feel like they came from the same hand.

## v0.3.0 in one paragraph

This release is the "transcription quality pack" — three orthogonal
improvements to how Whisper transcribes the user's audio:
1. **VAD pre-processing** (no UI, automatic in background)
2. **Vocabulary post-substitution** — exact-match replacement after
   transcription. Catches errors Whisper hallucinates despite the prompt.
   New UI: a table on the existing Vocabulario page.
3. **Custom prompt presets** — switchable Whisper-decoder prompts that
   bias the transcription toward a context (formal email vs casual
   WhatsApp vs technical code). Coexist with vocabulary. New UI: a
   dedicated page at `/main/presets`.

Both new UIs currently exist as functional but un-aesthetic drafts.

## Components needing design

### 1. Substitutions table — `src/routes/main/vocabulario/+page.svelte`

**Where it lives**: appended below the existing Vocabulario textarea card
on the same page. Both controls feed into transcription quality, so they
share a page; the user shouldn't have to navigate elsewhere.

**Function**: a list of `{ from, to, case_sensitive }` rules. After
each transcription Quill applies them as word-boundary regex replacements.
Use case: Whisper consistently transcribes "Mokia" instead of "Nokia"
even with vocabulary biasing → user adds a substitution and the brand
name is fixed for every future dictation.

**Current Svelte source** (the section to redesign — the vocabulary
textarea above stays):

```svelte
<h2 class="text-base-strong mt-10 text-[16px] font-semibold tracking-tight">Sustituciones</h2>
<p class="text-base-dim mt-1 text-[13px]">
  Reemplazo exacto post-transcripción. Útil para errores que el vocabulario no logra evitar — Whisper
  se obstina en escribir <span class="font-mono">"Mokia"</span> y vos querés <span class="font-mono">"Nokia"</span>.
</p>

<div class="border-hair bg-panel mt-4 overflow-hidden rounded-lg border">
  {#if subs.length === 0}
    <div class="p-6 text-center">
      <p class="text-base-mute text-[12.5px]">
        Sin sustituciones aún. Agregá una para empezar.
      </p>
    </div>
  {:else}
    <table class="w-full text-[12.5px]">
      <thead class="border-hair bg-elev border-b">
        <tr>
          <th>Reemplazar</th>
          <th>Por</th>
          <th>Aa</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        {#each subs as s, i (i)}
          <tr class="border-hair border-b last:border-b-0">
            <td><input type="text" placeholder="Mokia" value={s.from} ... /></td>
            <td><input type="text" placeholder="Nokia" value={s.to} ... /></td>
            <td><input type="checkbox" checked={s.case_sensitive} ... /></td>
            <td><button onclick={() => removeSub(i)}>×</button></td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}

  <div class="border-hair flex items-center justify-end border-t p-2">
    <button onclick={addSub}>+ Agregar</button>
  </div>
</div>
```

**Functional requirements**:
- Empty state: friendly "Sin sustituciones aún" placeholder + add button visible
- Each row: two equal-width text inputs (`from`, `to`), one small "Aa"
  toggle for case-sensitivity, a delete affordance
- Add button: at the bottom-right of the table card, small/secondary —
  not a primary CTA
- Inline editing: changes save automatically (debounced) as the user
  types. No "save" button.
- Validation: `from` is required (empty rules are silently dropped on
  save); `to` can be empty (means "delete the matched text"). No
  visible validation feedback unless the rule fails.

**Edge cases**:
- Long `from` text overflows the input → input scrolls horizontally
- 50+ rules → table scrolls vertically inside its card; card height capped
- Two rules with the same `from` are allowed (apply in order)

**Goals for the redesign**:
- Match the existing Vocabulario page's card styling (already polished)
- Make the "Aa" case-toggle visually explicit — currently a bare
  checkbox, easy to miss
- Make the row delete affordance discoverable but not noisy (currently
  a `×` that looks like cancel button)
- Keep the table dense — users may have 20+ rules

### 2. Presets master/detail page — `src/routes/main/presets/+page.svelte`

**Where it lives**: brand new page at `/main/presets`. Sidebar nav
already has the entry (currently reuses the Vocabulario icon as
placeholder — see component #3 below).

**Function**: lets the user manage prompt presets that bias Whisper's
decoder toward a context. Examples:
- "Email" preset: formal punctuation, complete sentences, "Estimado /
  Saludos cordiales" patterns
- "Casual" preset: contractions, lunfardo, "che", "dale"
- "Código" preset: snake_case identifiers, technical terms

The user marks one preset as **active** (or "Sin preset" for none); at
transcribe time the active preset's prompt is concatenated with the
global vocabulary and passed to Whisper as `--prompt`.

**Current Svelte source** (whole page — see
`src/routes/main/presets/+page.svelte` in the repo for the full file).
Layout:

```svelte
<div class="mx-auto flex max-w-[920px] gap-6 p-8">
  <!-- Left column: presets list -->
  <div class="border-hair bg-panel w-[260px] shrink-0 overflow-hidden rounded-lg border">
    <div class="border-hair flex items-center justify-between border-b px-3 py-2.5">
      <span>Presets</span>
      <button onclick={addCustomPreset}>+ Nuevo</button>
    </div>

    <button onclick={() => selectedId = null}>
      <span>Sin preset</span>
      <input type="radio" checked={activeId === null} onchange={() => setActive(null)} />
    </button>

    {#each presets as p}
      <button onclick={() => selectedId = p.id}>
        <div>
          <div>{p.name}</div>
          {#if p.builtin}<div>Built-in</div>{/if}
        </div>
        <input type="radio" checked={activeId === p.id} onchange={() => setActive(p.id)} />
      </button>
    {/each}
  </div>

  <!-- Right column: editor -->
  <div class="min-w-0 flex-1">
    <h1>Presets de prompt</h1>
    <p>Cada preset le da a Whisper un contexto distinto — formal para email...</p>

    {#if selected}
      <div>
        <label>
          Nombre
          <input type="text" value={selected.name} disabled={selected.builtin} />
        </label>
        <label>
          Prompt
          <textarea value={selected.prompt}>~880 char limit</textarea>
        </label>
        <button onclick={() => setActive(...)}>
          {activeId === selected.id ? '✓ Preset activo' : 'Marcar como activo'}
        </button>
        {#if !selected.builtin}
          <button onclick={deleteSelected}>Eliminar</button>
        {/if}
      </div>
    {/if}
  </div>
</div>
```

**Built-in presets** (shipped, can be edited but not deleted):
- General — neutral baseline
- Código — code-style technical bias
- Email — formal correspondence bias
- Casual — informal / lunfardo bias

**Custom presets** (user-created): full lifecycle — create, rename, edit
prompt, delete.

**Functional requirements**:
- Master list (left): one row per preset + "Sin preset" pseudo-row at
  top. Each row has a radio button to mark **active** (only one active
  at a time). Active state and selection state are independent: a user
  may select a row to edit its prompt without making it active.
- Detail (right): shows the selected preset's name (read-only for
  built-ins) and prompt textarea (~880 char hint near the bottom).
  "Marcar como activo" / "✓ Preset activo" toggle, plus a "Eliminar"
  link for non-built-in presets only.
- Empty selection state: when user clicks "Sin preset" pseudo-row, the
  detail panel becomes a "no preset selected" empty card.
- "+ Nuevo" button at the top of the master list creates a "Custom N"
  preset and selects it for editing.
- All saves are immediate (no save button) — debounced for the textarea.

**Edge cases**:
- 20+ user-created presets → master list scrolls vertically; detail panel
  height matches viewport
- Built-in name is locked (input disabled) but prompt is editable
- Deleting the active preset → falls back to "Sin preset"; the radio
  group reflects this immediately

**Goals for the redesign**:
- Distinguish built-in from user presets visually (currently a small
  "Built-in" subtitle)
- Make the active-state radio button feel more substantial — it's the
  primary action of the page
- Active toggle in the detail panel: currently a text button that
  changes label between "Marcar como activo" and "✓ Preset activo".
  Could be a more visual switch / pill / segmented control.
- The page is dense; whitespace matters

### 3. Sidebar nav icon for "Presets"

Currently the new "Presets" entry in the sidebar reuses the Vocabulario
icon as a placeholder (see `src/lib/components/ui/Sidebar.svelte`):

```typescript
{ id: 'presets', label: 'Presets', Icon: IconVocab },
```

We need a distinct icon. Mental shortcut: a "preset" is a saved
configuration / template. Possible directions: a stack of cards, a tag,
a stamp, a layered template glyph.

Style: matches the existing Lucide-style sidebar icons (1px stroke,
currentColor, 16×16 default size). See `src/lib/components/ui/icons/`
for the existing set.

## Existing design system reference

Don't reinvent these — read directly from the existing project:

- **CSS tokens**: `src/app.css` for colors, spacing, fonts. Uses
  `oklch()` palette with `--bg`, `--panel`, `--elev`, `--border-hair`,
  `--text-strong`, `--text-dim`, `--text-mute`, `--accent`, etc.
- **Tailwind utility classes**: the project uses class-based aliases
  like `bg-panel`, `bg-elev`, `border-hair`, `text-base-strong`,
  `text-base-dim`, `text-base-mute` that map to those tokens
- **Existing primitives**: `src/lib/components/ui/` includes
  `Accordion`, `Button`, `Toggle`, `Segmented`, `Dropdown`,
  `RadioCard`, `Slider`, `KeyCapture`, `KeyCap`, `Pill`, etc. Reuse
  before introducing new primitives.
- **Reference HTML**: `design/reference/project/Quill - App Screens.html`
  is the original Claude Design output for the 8 existing screens. It's
  the source of truth for spacing, type ramp, card styling, hover
  treatments. The new components should look like missing siblings of
  those screens, not new ones.

## Deliverables

For each component, an HTML/CSS mockup matching the format of the
existing `design/reference/project/Quill - App Screens.html` — i.e. a
self-contained prototype showing the component in:

1. **Default state** with realistic sample data (3-5 substitution rules,
   the 4 built-in presets + 2 custom)
2. **Empty state** (no substitutions yet, no preset selected)
3. **Hover / focus / disabled states** for interactive elements
4. **Light theme + dark theme** if the existing bundle supports both
5. **At least one icon variant** for the Presets sidebar entry

Spanish copy throughout (matches the rest of the app).

## Iteration model

Once Claude Design produces the bundle:
1. User exports it and merges into `design/reference/project/`
2. Coding agent (me) reads the new HTML and rebuilds the Svelte
   components to match the new visual treatment
3. Functional behavior + Tauri commands stay as-is — only the markup
   and CSS change

## Files to inspect before designing

The fastest way to understand the existing aesthetic:

- `design/reference/project/Quill - App Screens.html` — full original
  bundle (the 8 existing screens)
- `src/routes/main/vocabulario/+page.svelte` — current Vocabulario page
  (the textarea card + "Cómo funciona" callout are well-styled; the
  Substitutions section below them is the new draft)
- `src/routes/main/presets/+page.svelte` — current draft of the new
  Presets page
- `src/lib/components/ui/Sidebar.svelte` — sidebar nav structure for
  context on where the Presets icon sits
- `src/app.css` — design tokens
