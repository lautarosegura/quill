<script lang="ts">
	import { onMount } from 'svelte';
	import { Button, Dropdown, Toggle } from '$lib/components/ui';
	import * as api from '$lib/ipc/commands';
	import { config } from '$lib/stores/config.svelte';
	import type { LlmModelInfo, LlmProvider } from '$lib/types';
	import ProviderKeyPanel from './ProviderKeyPanel.svelte';
	import PolishPreview from './PolishPreview.svelte';

	const PROVIDERS: LlmProvider[] = ['groq', 'anthropic', 'openai'];
	const PROVIDER_LABELS: Record<LlmProvider, string> = {
		groq: 'Groq',
		anthropic: 'Anthropic',
		openai: 'OpenAI'
	};
	const DEFAULT_SYSTEM_PROMPT =
		'Limpiá esta transcripción dictada por voz: remové muletillas (eh, mm, este, o sea), agregá puntuación si falta, y unificá mayúsculas. NO cambies palabras, NO cambies el sentido, NO traduzcas. Devolvé sólo el texto limpio, sin comentarios ni explicaciones.';
	const PROMPT_MAX_CHARS = 2000;

	// Per-provider masked-key state. Drives the segmented "key configured" dot
	// and gates the Probar button in the preview.
	let maskedKeys = $state<Partial<Record<LlmProvider, string | null>>>({});
	let modelsByProvider = $state<Partial<Record<LlmProvider, LlmModelInfo[]>>>({});
	let promptDraft = $state('');
	let promptDirty = $state(false);

	const enabled = $derived<boolean>(config.value?.llm_polish_enabled ?? false);
	const activeProvider = $derived<LlmProvider>(config.value?.llm_polish_provider ?? 'groq');
	const activeModelId = $derived<string>(
		config.value?.llm_polish_models?.[activeProvider] ?? defaultModelId(activeProvider)
	);
	const activeModelInfo = $derived<LlmModelInfo | undefined>(
		modelsByProvider[activeProvider]?.find((m) => m.id === activeModelId)
	);
	const activeKeyMasked = $derived<string | null>(maskedKeys[activeProvider] ?? null);
	const keyConfigured = $derived<boolean>(activeKeyMasked != null);

	const promptCharCount = $derived(promptDraft.length);
	const promptOverflow = $derived(promptCharCount > PROMPT_MAX_CHARS);

	function defaultModelId(p: LlmProvider): string {
		switch (p) {
			case 'groq':
				return 'llama-3.3-70b-versatile';
			case 'anthropic':
				return 'claude-haiku-4-5-20251001';
			case 'openai':
				return 'gpt-4o-mini';
		}
	}

	const modelDropdownOptions = $derived(
		(modelsByProvider[activeProvider] ?? []).map((m) => ({
			value: m.id,
			label: m.recommended ? `${m.display_name} · Recomendado` : m.display_name
		}))
	);

	onMount(async () => {
		// Pull all three masked keys + all three model lists upfront so the
		// segmented control dots, the model dropdown, and switching providers
		// are instant.
		await Promise.all(
			PROVIDERS.map(async (p) => {
				try {
					maskedKeys[p] = await api.getLlmPolishKeyMasked(p);
				} catch {
					maskedKeys[p] = null;
				}
				try {
					modelsByProvider[p] = await api.listLlmPolishModels(p);
				} catch {
					modelsByProvider[p] = [];
				}
			})
		);

		promptDraft = config.value?.llm_polish_system_prompt ?? DEFAULT_SYSTEM_PROMPT;
	});

	// Sync the textarea draft to config any time config changes (e.g. on first
	// load) but only when the user hasn't typed anything dirty yet.
	$effect(() => {
		if (promptDirty) return;
		const fromConfig = config.value?.llm_polish_system_prompt;
		if (fromConfig != null && fromConfig !== promptDraft) {
			promptDraft = fromConfig;
		}
	});

	function setEnabled(v: boolean) {
		config.set('llm_polish_enabled', v);
	}

	function pickProvider(p: LlmProvider) {
		config.set('llm_polish_provider', p);
	}

	function pickModel(id: string) {
		const next = { ...(config.value?.llm_polish_models ?? {}), [activeProvider]: id };
		config.set('llm_polish_models', next);
	}

	function onPromptInput(e: Event) {
		promptDraft = (e.currentTarget as HTMLTextAreaElement).value;
		promptDirty = true;
		// Debounced save through config store.
		config.setDebounced('llm_polish_system_prompt', promptDraft, 600);
	}

	function restoreDefaultPrompt() {
		promptDraft = DEFAULT_SYSTEM_PROMPT;
		promptDirty = true;
		config.set('llm_polish_system_prompt', DEFAULT_SYSTEM_PROMPT);
	}

	function onKeyChanged(provider: LlmProvider, masked: string | null) {
		maskedKeys[provider] = masked;
	}
</script>

<div class="flex flex-col gap-5 py-2">
	<!-- Master toggle row -->
	<div class="flex items-center gap-3">
		<div class="min-w-0 flex-1">
			<div class="text-base-strong text-[12.5px] font-medium leading-tight">
				Habilitar pulido con IA
			</div>
			<div class="text-base-mute mt-0.5 text-[11.5px]" style="text-wrap: pretty;">
				Después de transcribir, una pasada del LLM remueve muletillas y normaliza la puntuación. No
				cambia el sentido.
			</div>
		</div>
		<Toggle value={enabled} onChange={setEnabled} />
	</div>

	{#if enabled}
		<!-- Provider switcher -->
		<div class="flex flex-col gap-2">
			<span class="label-eyebrow">Proveedor</span>
			<div class="flex items-center justify-between gap-3 flex-wrap">
				<div class="seg" role="tablist">
					{#each PROVIDERS as p (p)}
						<button
							type="button"
							class="{p} {activeProvider === p ? 'is-active' : ''}"
							onclick={() => pickProvider(p)}
							role="tab"
							aria-selected={activeProvider === p}
						>
							<span class="pdot"></span>
							{PROVIDER_LABELS[p]}
							{#if maskedKeys[p]}
								<span class="pkstate"></span>
							{/if}
						</button>
					{/each}
				</div>
				<span class="text-base-mute text-[11px] font-mono">
					{Object.values(maskedKeys).filter(Boolean).length} / 3 con clave
				</span>
			</div>
			<div class="text-base-mute text-[11px]" style="text-wrap: pretty;">
				Cada proveedor guarda su propia clave en el keychain del sistema y su modelo preferido.
			</div>
		</div>

		<!-- API key panel for the active provider -->
		<ProviderKeyPanel
			provider={activeProvider}
			masked={activeKeyMasked}
			onMaskedChange={(m) => onKeyChanged(activeProvider, m)}
		/>

		<!-- Model picker -->
		<div class="grid grid-cols-[1fr_auto] gap-3 items-end">
			<div class="flex flex-col gap-2">
				<span class="label-eyebrow">Modelo</span>
				{#if keyConfigured && modelDropdownOptions.length > 0}
					<Dropdown
						value={activeModelId}
						options={modelDropdownOptions}
						onChange={pickModel}
					/>
				{:else}
					<div class="qselect-disabled">
						<select disabled>
							<option>Configurá la clave primero</option>
						</select>
					</div>
				{/if}
			</div>
			{#if activeModelInfo?.blurb}
				<div class="text-base-mute text-[11.5px] pb-2" style="max-width: 240px; text-wrap: pretty;">
					{activeModelInfo.blurb}
				</div>
			{/if}
		</div>

		<!-- System prompt -->
		<div class="flex flex-col gap-2">
			<div class="flex items-center justify-between">
				<span class="label-eyebrow">Prompt del sistema</span>
				<span
					class="font-mono text-[10.5px]"
					style="color: {promptOverflow ? 'oklch(0.72 0.18 25)' : 'var(--text-mute)'};"
				>
					{promptCharCount} / {PROMPT_MAX_CHARS}
				</span>
			</div>
			<textarea
				class="prompt-area"
				rows="7"
				value={promptDraft}
				oninput={onPromptInput}
			></textarea>
			<div class="flex items-center justify-between">
				<p class="text-base-mute text-[11px]" style="text-wrap: pretty;">
					Auto-guardado · debounce 600ms.
				</p>
				<Button variant="ghost" size="sm" onclick={restoreDefaultPrompt}>Restaurar default</Button>
			</div>
		</div>

		<!-- Live preview -->
		<PolishPreview
			provider={activeProvider}
			modelDisplayName={activeModelInfo?.display_name ?? activeModelId}
			{keyConfigured}
		/>
	{/if}
</div>

<style>
	.label-eyebrow {
		font-size: 10px;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		font-weight: 600;
		color: var(--text-mute);
	}

	.seg {
		display: inline-flex;
		align-items: center;
		padding: 3px;
		border: 1px solid var(--border);
		background: var(--bg-elev);
		border-radius: 8px;
		gap: 2px;
	}
	.seg button {
		height: 28px;
		padding: 0 14px;
		border-radius: 6px;
		font-size: 12px;
		font-weight: 500;
		color: var(--text-mute);
		display: inline-flex;
		align-items: center;
		gap: 6px;
		transition: background 120ms ease, color 120ms ease;
		cursor: pointer;
		background: transparent;
		border: none;
	}
	.seg button:hover {
		color: var(--text-dim);
		background: var(--hover);
	}
	.seg button.is-active {
		background: var(--bg-panel);
		color: var(--text);
		box-shadow: 0 1px 0 oklch(1 0 0 / 0.04) inset, 0 1px 2px oklch(0 0 0 / 0.25);
	}
	.seg button .pdot {
		width: 7px;
		height: 7px;
		border-radius: 999px;
		background: var(--text-mute);
		opacity: 0.5;
	}
	.seg button.is-active .pdot {
		opacity: 1;
	}
	.seg button.is-active.groq .pdot {
		background: oklch(0.78 0.16 35);
	}
	.seg button.is-active.anthropic .pdot {
		background: oklch(0.72 0.13 40);
	}
	.seg button.is-active.openai .pdot {
		background: oklch(0.78 0.13 165);
	}
	.seg button .pkstate {
		width: 5px;
		height: 5px;
		border-radius: 999px;
		background: oklch(0.75 0.16 155);
		margin-left: 2px;
	}

	.prompt-area {
		width: 100%;
		background: var(--bg);
		border: 1px solid var(--border-strong);
		border-radius: 8px;
		padding: 12px 14px;
		font-family: 'JetBrains Mono', ui-monospace, monospace;
		font-size: 12.5px;
		line-height: 1.55;
		color: var(--text);
		resize: vertical;
		outline: none;
		transition: border-color 120ms ease, box-shadow 120ms ease;
	}
	.prompt-area:focus {
		border-color: color-mix(in oklch, var(--accent) 50%, var(--border-strong));
		box-shadow: 0 0 0 3px var(--accent-ring);
	}

	.qselect-disabled {
		position: relative;
		width: 100%;
		opacity: 0.55;
	}
	.qselect-disabled select {
		width: 100%;
		height: 34px;
		padding: 0 32px 0 12px;
		background: var(--bg-elev);
		border: 1px solid var(--border-strong);
		border-radius: 7px;
		color: var(--text-mute);
		font-size: 12.5px;
		appearance: none;
		outline: none;
		cursor: not-allowed;
	}
</style>
