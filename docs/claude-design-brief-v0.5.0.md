# Claude Design brief — Quill v0.5.0 LLM polish components

## What this is

A handoff brief in the **opposite direction** of the original Quill bundle.
The original `design/reference/project/` was Claude Design → coding agent.
This brief is coding agent → Claude Design: a new Settings section was
added in v0.5.0 with deliberately-minimal styling, and we need it
redesigned to match the existing aesthetic before the release.

Format mirrors `docs/claude-design-brief-v0.3.0.md`.

## Project context

Quill is a voice-to-text dictation app for Windows / Linux / macOS, built
with Tauri 2 + Svelte 5. The visual system is already established in the
8 existing screens — see `design/reference/project/Quill - App Screens.html`
for the full HTML/CSS source of every page (Historial, Settings,
Vocabulario, Modelos, Uso, Wizard, Overlay, Main shell). The new
components must feel like they came from the same hand.

## v0.5.0 in one paragraph

This release adds an optional **LLM cleanup stage** that runs after
Whisper transcription. The user picks a cloud LLM provider (**Groq,
Anthropic, or OpenAI**), enters their API key, picks a model, edits the
system prompt that drives the cleanup, and gets a live preview to test
the prompt. Off by default — opt-in. When enabled, it removes muletillas
("eh", "mm", "o sea"), normalizes punctuation, doesn't change meaning.
Three providers ship in v0.5.0; local LLM via llama.cpp is deferred.

The whole feature lives inside **one new Settings accordion** that the
user can open, configure, and close — no new pages, no separate
navigation. The accordion contains a small live-preview tester at the
bottom for "see what my settings actually do" experimentation.

## Components needing design

### 1. Settings "Pulido con IA" accordion — `src/routes/main/settings/+page.svelte`

**Where it lives**: third accordion from the top, after "Motor de
transcripción" (currently `defaultOpen`) and before "Modelo local".
Reuses the existing `<Accordion>` component (`src/lib/components/ui/Accordion.svelte`)
— same icon-tinted box, same chevron, same fade-in. The icon to use is
`IconSparkles` (already in the design system, used elsewhere for AI-touched
features).

**Function**: lets the user enable an LLM cleanup pass on every
transcription. Inside the accordion the user configures:
1. The master toggle (enabled / disabled)
2. The provider (one of 3: Groq, Anthropic, OpenAI)
3. The API key for the active provider
4. The model for the active provider
5. The system prompt
6. Live preview tester (next component, see Brief 2)

The accordion **summary line** (one-liner shown next to the title when
the accordion is closed) must show current state at a glance. Examples:
- `Desactivado`
- `Groq · llama-3.3-70b-versatile`
- `Anthropic · Claude Haiku 4.5`
- `OpenAI · gpt-4o-mini`

**Current Svelte source** (the placeholder section to be redesigned):

```svelte
<!-- ACCORDION: PULIDO CON IA — placeholder, redesign me -->
<Accordion title="Pulido con IA" summary={polishSummary} icon={IconSparkles}>
  <Toggle
    value={config.value?.llm_polish_enabled ?? false}
    label="Habilitar pulido con IA"
    onChange={(v) => api.setConfig({ llm_polish_enabled: v })}
  />

  {#if config.value?.llm_polish_enabled}
    <!-- Provider picker — three options visible, only one active -->
    <Segmented
      options={[
        { value: 'groq', label: 'Groq' },
        { value: 'anthropic', label: 'Anthropic' },
        { value: 'openai', label: 'OpenAI' }
      ]}
      value={config.value?.llm_polish_provider ?? 'groq'}
      onChange={(p) => api.setConfig({ llm_polish_provider: p })}
    />

    <!-- API key panel for the active provider — three states (saved/editing/empty) -->
    <div class="border-hair bg-panel rounded-lg border p-4">
      {#if keyMasked && !editingKey}
        <span style="color: oklch(0.75 0.16 155)">● Configurada</span>
        <span class="font-mono">{keyMasked}</span>
        <Button size="sm" variant="secondary" onclick={() => editingKey = true}>Cambiar clave</Button>
        <Button size="sm" variant="ghost" onclick={deleteKey}>Borrar</Button>
      {:else}
        <PasswordInput value={keyInput} onChange={(v) => keyInput = v} placeholder="sk-..." />
        <Button size="sm" variant="secondary" onclick={testKey} disabled={!keyInput}>Probar</Button>
        <Button size="sm" variant="primary" onclick={saveKey}>Guardar</Button>
        {#if keyMasked}<Button size="sm" variant="ghost" onclick={() => editingKey = false}>Cancelar</Button>{/if}
      {/if}
      {#if testMessage}<div class="test-feedback">{testMessage}</div>{/if}
    </div>

    <!-- Model picker, scoped to active provider -->
    <Dropdown
      label="Modelo"
      value={modelForActiveProvider}
      options={modelOptions}
      onChange={(m) => updateModel(m)}
    />
    <p class="text-base-mute">{modelBlurb}</p>

    <!-- System prompt editor -->
    <div class="prompt-section">
      <label>Prompt del sistema</label>
      <textarea
        value={config.value?.llm_polish_system_prompt}
        oninput={(e) => debouncedUpdatePrompt(e.currentTarget.value)}
      ></textarea>
      <span>{promptCharCount} / 2000</span>
      <Button size="sm" variant="ghost" onclick={resetPrompt}>Restaurar default</Button>
    </div>

    <!-- Live preview pane — see Brief 2 -->
    <PolishPreview {systemPrompt} {provider} {model} />
  {/if}
</Accordion>
```

**Functional requirements**:

- **Summary line** ("Desactivado" / "Groq · llama-3.3-70b-versatile") matches existing Settings summaries — same typography, same color (`text-base-dim` on existing cards).
- **Toggle at top** of accordion enables/disables the feature. When off, the rest of the accordion content stays mounted but visually de-emphasized OR collapses to nothing — designer's choice. (Current placeholder: simple `{#if enabled}` collapse.)
- **Provider switcher**: when user changes provider, the key panel + model dropdown re-render scoped to that provider. Each provider has its OWN API key (stored in OS keychain under `groq_llm_key`, `anthropic_llm_key`, `openai_llm_key`) and its OWN preferred model (stored in `config.llm_polish_models[provider]`). Switching providers is instant — it's a config write, not an API call.
- **API key panel** mirrors the EXISTING Groq key panel verbatim (look at the "Motor de transcripción" → Groq config block in the same file for visual reference). Three states:
  - **Empty** (no key for this provider): PasswordInput + Probar (secondary) + Guardar (primary)
  - **Saved** (key in keychain): green dot "● Configurada" + masked last-4 (e.g. `sk-...abc4`) + Cambiar clave (secondary) + Borrar (ghost)
  - **Editing** (user clicked Cambiar clave): PasswordInput + Probar + Guardar + Cancelar (ghost)
  - **Test feedback** after Probar: green inline message ("Clave válida · 6 modelos disponibles") or red inline message ("Clave inválida"), shown below the buttons.
- **Model dropdown**: lists models for the active provider with the recommended one first labeled "Recomendado". Each model has a 1-line blurb (e.g. "Rápido y barato. Recomendado para uso diario.") shown next to or below the dropdown.
- **System prompt textarea**: 6-8 visible rows, max 2000 chars with a counter (turns red at overflow), "Restaurar default" button to revert to the factory prompt, debounced auto-save (no Save button) — same pattern as the existing prompt-presets editor at `src/routes/main/presets/+page.svelte`.
- **Live preview pane** (Brief 2 below) at the bottom.

**Edge cases**:

- **No key for the selected provider**: show inline hint "Necesitás configurar la clave para usar este proveedor" with the key panel below. Model dropdown is disabled until key is saved.
- **Key invalid (401 from Probar)**: red feedback message below the Probar button; Guardar stays enabled (user can save anyway, but next Probar will likely fail).
- **Provider switched while a polish call is in-flight**: the in-flight preview call is cancelled (frontend AbortController). UI shows nothing weird — just the new provider's panel.
- **Long system prompt (>2000 chars)**: counter turns red, no truncation, "Restaurar default" button still works.
- **All three providers configured**: switching is instant, no API call. Each provider's key + model preference stays in its own slot.
- **First-time user opens the accordion (toggle off, no keys, no model preferences)**: default provider is Groq, default model is `llama-3.3-70b-versatile`, system prompt has the factory default in the textarea.

**Goals for the redesign**:

- Match the existing Settings accordion aesthetic (icon-tinted box, dense vertical rhythm, OKLCH accent palette, same `--accent-soft`/`--accent` tokens).
- Make the "Desactivado" vs active distinction visually obvious without screaming. The toggle at the top is the primary affordance.
- The 3-provider switcher should feel cohesive — the user might have all three configured. They shouldn't get visual whiplash switching providers in this section. Consider whether tabs, segmented control, or a sub-accordion-per-provider works best aesthetically. (Current placeholder: `Segmented` with 3 options.)
- Make the API key UX for THREE providers feel cohesive — same panel shape, same buttons, same feedback affordances, just scoped to the active provider.
- Keep the system prompt textarea calm — it's a power-user field, not a CTA. Default text styling, no syntax highlighting.
- The accordion will be ~720px wide inside the Settings page. Plan for that width. No need to handle narrower viewports — Settings doesn't get narrow.

### 2. Live preview pane — `src/lib/components/settings/PolishPreview.svelte` (new file)

**Where it lives**: at the bottom of the "Pulido con IA" accordion, only
visible when the toggle is on AND the active provider has a key
configured. New component file, called from inside the Settings page as
`<PolishPreview {systemPrompt} {provider} {model} />`.

**Function**: lets the user paste sample text, hit "Probar", and see
the polished output side-by-side with the original. Surfaces latency
and token counts. Surfaces errors HERE (rate limit, network, etc.)
instead of in the main pipeline — the main pipeline degrades silently
to raw transcription on any failure; the preview is the place to
diagnose. This is the "I want to see what my prompt + model + provider
combo actually does" workshop area.

**Current Svelte source** (placeholder, redesign me):

```svelte
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { Button, Spinner } from '$lib/components/ui';

  let { systemPrompt, provider, model } = $props();
  let sample = $state('eh hola este... como te va? mm queria preguntarte si...');
  let polished = $state<string | null>(null);
  let latency = $state<number | null>(null);
  let inputTokens = $state<number | null>(null);
  let outputTokens = $state<number | null>(null);
  let busy = $state(false);
  let error = $state<string | null>(null);

  async function runPreview() {
    busy = true; error = null; polished = null;
    try {
      const result = await invoke<PolishPreview>('test_llm_polish', { text: sample });
      polished = result.polished;
      latency = result.latency_ms;
      inputTokens = result.input_tokens;
      outputTokens = result.output_tokens;
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<div class="preview-card">
  <div class="preview-header">
    <span>Vista previa</span>
    <span class="text-base-mute">Probá tu prompt con un texto de muestra</span>
  </div>

  <div class="preview-grid">
    <!-- INPUT side -->
    <div class="preview-side">
      <label>Texto crudo</label>
      <textarea
        value={sample}
        oninput={(e) => sample = e.currentTarget.value}
        placeholder="Pegá una transcripción cruda…"
      ></textarea>
    </div>

    <!-- OUTPUT side -->
    <div class="preview-side">
      <label>Texto pulido</label>
      <div class="preview-output">
        {#if busy}
          <Spinner /> <span>Procesando…</span>
        {:else if error}
          <div class="error">{error}</div>
        {:else if polished}
          {polished}
        {:else}
          <span class="text-base-mute">Apretá "Probar" para ver el resultado</span>
        {/if}
      </div>
    </div>
  </div>

  <div class="preview-footer">
    <Button size="sm" variant="primary" onclick={runPreview} disabled={busy || !sample}>
      Probar
    </Button>
    {#if latency}
      <span class="text-base-mute font-mono">{latency}ms</span>
      {#if inputTokens && outputTokens}
        <span class="text-base-mute font-mono">{inputTokens} → {outputTokens} tokens</span>
      {/if}
    {/if}
  </div>
</div>
```

**Functional requirements**:

- **Two-pane layout**, side-by-side on the Settings width (~720px). Stacked vertically can also work if it looks cleaner.
- **Sample default text**: a short Spanish transcription with classic muletillas, e.g. `"eh hola este... como te va? mm queria preguntarte si tenes tiempo el viernes para juntarnos a charlar de eso que me decias"`.
- **Probar button**: triggers the polish call. Shows spinner during call (typical 200-2000ms latency).
- **After response**: render the polished text as plain prose (not monospace — match Settings body copy).
- **Latency + token counts**: render below the button as quiet muted metadata. Token counts are optional (some providers don't return them).
- **Error state**: red text in the output side. No retry button — the user just hits Probar again.
- **Disabled state**: when sample empty, when busy, or when no key is configured for the active provider (in which case show "Configurá la clave para probar").

**Edge cases**:

- **Long sample (>500 chars)**: textarea scrolls vertically inside its bounds. Output area also scrolls.
- **Network failure**: error message says "No se pudo conectar — verificá tu conexión".
- **401 / Unauthorized**: "Clave inválida — revisá tu API key".
- **429 / Rate limit**: "Rate limit — esperá unos segundos y probá de nuevo".
- **Polish returns same text (provider didn't change anything)**: render with subtle "(sin cambios)" hint below the polished side.
- **Provider/model changed mid-preview**: previous result stays visible until next Probar — don't auto-refresh.

**Goals for the redesign**:

- Make this feel like a developer playground inside Settings — slightly different visual weight from the rest of the accordion (it's a tool, not a setting).
- The before→after comparison should be visually pleasant. A subtle highlight on words that changed would be delightful (optional — if too complex, plain text is fine).
- The token + latency footer should be quiet metadata, not a feature in itself. Right-aligned next to the Probar button works well.
- Minimum vertical space when the accordion is open — no huge whitespace. The whole preview pane shouldn't take more than ~250px tall.
- Don't reinvent textarea styling — match the existing prompt-presets textarea (`src/routes/main/presets/+page.svelte`).

## Design system primitives available

These already exist in `src/lib/components/ui/index.ts` and should be
reused — don't reinvent:

- `Accordion` — the shell
- `Toggle` — the master enable
- `Segmented` — the 3-provider switcher (or replace with custom design)
- `Dropdown` — the model picker
- `PasswordInput` — the API key input with show/hide
- `Button` (variants: `primary`, `secondary`, `ghost`)
- `Spinner` — loading state
- `Icon` + the existing icon set including `IconSparkles`

## Tokens / colors used elsewhere

- `--accent`, `--accent-soft` — purple primary
- `--text-base-strong`, `--text-base-dim`, `--text-base-mute` — text hierarchy
- `--bg-app`, `--bg-panel`, `--bg-elev` — background hierarchy
- `--border-hair` — 1px dividers
- Success green: `oklch(0.75 0.16 155)`
- Error red: `oklch(0.72 0.18 25)`

## What we'll do with the bundle

When you (Claude Design) hand back the redesigned Svelte file (or HTML/CSS
mock + Svelte target), the coding agent will:

1. Replace the placeholder accordion at the marked location in
   `src/routes/main/settings/+page.svelte`.
2. Create the new `PolishPreview.svelte` component file.
3. Wire to the Tauri commands (already implemented backend-side):
   `get_llm_polish_key_masked`, `set_llm_polish_key`,
   `delete_llm_polish_key`, `test_llm_polish_key`, `test_llm_polish`,
   `list_llm_polish_models`.
4. Hook config saves through the existing `api.setConfig({ … })` helper.
5. Type the props strictly against the existing TypeScript types.
6. Hand back the working Settings page for visual QA.

## Questions for Claude Design

If anything in this brief is ambiguous, ask before designing. The
common pitfalls in past handoffs were:

- Reusing the wrong icon (please use `IconSparkles` for the accordion title icon)
- Inventing new color tokens instead of using `--accent`, etc.
- Designing for a width other than 720px
- Forgetting that the accordion content is mounted/unmounted by the parent
  (so any in-component state lives only while the accordion is open)
