<script lang="ts">
	import { Button } from '$lib/components/ui';
	import * as api from '$lib/ipc/commands';
	import type { LlmProvider, PolishPreviewResult } from '$lib/types';
	import { wordDiff, type DiffPart } from '$lib/utils/diff';

	interface Props {
		/** Provider currently active in Settings — shown in the header. */
		provider: LlmProvider;
		/** Display name of the model in use, for the header label. */
		modelDisplayName: string;
		/** Whether the active provider has a key configured. When false the
		 *  Probar button is disabled and a hint is shown instead of a result. */
		keyConfigured: boolean;
	}

	let { provider, modelDisplayName, keyConfigured }: Props = $props();

	const providerLabels: Record<LlmProvider, string> = {
		groq: 'Groq',
		anthropic: 'Anthropic',
		openai: 'OpenAI'
	};

	const SAMPLE_DEFAULT =
		'eh hola este... como te va? mm queria preguntarte si tenes tiempo el viernes para juntarnos a charlar de eso que me decias el otro dia, viste';

	let sample = $state(SAMPLE_DEFAULT);
	let result = $state<PolishPreviewResult | null>(null);
	let busy = $state(false);
	let errorMsg = $state<string | null>(null);
	let highlightChanges = $state(true);

	const diffParts = $derived.by<DiffPart[] | null>(() => {
		if (!result) return null;
		if (!highlightChanges) return null;
		return wordDiff(result.original, result.polished);
	});

	const noChange = $derived.by(() => {
		if (!result) return false;
		return result.original.trim() === result.polished.trim();
	});

	async function runPreview() {
		if (!sample.trim() || busy) return;
		busy = true;
		errorMsg = null;
		result = null;
		try {
			const r = await api.testLlmPolish(sample);
			result = r;
		} catch (e: unknown) {
			errorMsg = e instanceof Error ? e.message : String(e);
		} finally {
			busy = false;
		}
	}

	function loadSample() {
		sample = SAMPLE_DEFAULT;
	}

	function tokensLine(): string | null {
		if (!result) return null;
		if (result.input_tokens == null || result.output_tokens == null) return null;
		return `${result.input_tokens} → ${result.output_tokens} tokens`;
	}
</script>

<div class="preview-card">
	<div class="header">
		<div class="title">
			<svg
				width="12"
				height="12"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.75"
				stroke-linecap="round"
				stroke-linejoin="round"
				class="title-icon"
			>
				<path d="M4 12h4l3-8 4 16 3-8h2" />
			</svg>
			<span class="title-text">Probá tu prompt</span>
			{#if keyConfigured}
				<span class="title-meta">— {providerLabels[provider]} · {modelDisplayName}</span>
			{:else}
				<span class="title-meta">Sandbox · no afecta tu pipeline</span>
			{/if}
		</div>
		<button type="button" class="btn-ghost-mini" onclick={loadSample} title="Cargar muestra">
			<svg
				width="11"
				height="11"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.75"
				stroke-linecap="round"
			>
				<path d="m15 18-6-6 6-6" />
			</svg>
			Cargar muestra
		</button>
	</div>

	<div class="preview-grid">
		<div class="preview-side">
			<div class="side-label">Texto crudo · {sample.length} chars</div>
			<textarea
				class="prose-area"
				rows="5"
				value={sample}
				oninput={(e) => (sample = e.currentTarget.value)}
				placeholder="Pegá una transcripción cruda…"
				disabled={busy}
			></textarea>
		</div>

		<div class="preview-arrow">
			<svg
				width="14"
				height="14"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.75"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<path d="M5 12h14M13 6l6 6-6 6" />
			</svg>
		</div>

		<div class="preview-side">
			<div class="side-label">
				Texto pulido
				{#if result}
					<span class="side-meta">· {result.polished.length} chars{noChange ? ' · (sin cambios)' : ''}</span>
				{/if}
			</div>
			<div class="preview-output" class:error={errorMsg}>
				{#if busy}
					<div class="busy">
						<svg
							width="14"
							height="14"
							viewBox="0 0 24 24"
							fill="none"
							class="spin"
						>
							<circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="2.5" opacity="0.25" />
							<path
								d="M21 12a9 9 0 0 0-9-9"
								stroke="currentColor"
								stroke-width="2.5"
								stroke-linecap="round"
							/>
						</svg>
						<span>Procesando…</span>
					</div>
				{:else if errorMsg}
					<div class="error-body">
						<svg
							width="13"
							height="13"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="1.75"
							stroke-linecap="round"
							stroke-linejoin="round"
							class="error-icon"
						>
							<circle cx="12" cy="12" r="9" />
							<path d="M12 8v5" />
							<path d="M12 16h.01" />
						</svg>
						<div class="error-text">{errorMsg}</div>
					</div>
				{:else if result && diffParts}
					{#each diffParts as part, i (i)}
						{#if part.kind === 'add'}
							<span class="diff-add">{part.text}</span>
						{:else if part.kind === 'rem'}
							<span class="diff-rem">{part.text}</span>
						{:else}
							<span>{part.text}</span>
						{/if}
					{/each}
				{:else if result}
					{result.polished}
				{:else if !keyConfigured}
					<span class="placeholder">Configurá la clave para probar.</span>
				{:else}
					<span class="placeholder">Apretá "Probar" para ver el resultado.</span>
				{/if}
			</div>
		</div>
	</div>

	<div class="preview-footer">
		<Button
			variant="primary"
			size="sm"
			onclick={runPreview}
			disabled={busy || !sample.trim() || !keyConfigured}
		>
			Probar
		</Button>

		{#if result}
			<span class="preview-meta">{result.latency_ms} ms</span>
			{#if tokensLine()}
				<span class="preview-meta">·</span>
				<span class="preview-meta">{tokensLine()}</span>
			{/if}
		{:else if errorMsg}
			<span class="ml-auto pill-err"><span class="dot"></span>error</span>
		{:else if busy}
			<span class="preview-meta">esperando respuesta…</span>
		{:else}
			<span class="preview-meta">—</span>
		{/if}

		<label class="ml-auto highlight-toggle">
			<input
				type="checkbox"
				bind:checked={highlightChanges}
				disabled={!result}
			/>
			Resaltar cambios
		</label>
	</div>
</div>

<style>
	.preview-card {
		border: 1px dashed var(--border-strong);
		background: color-mix(in oklch, var(--accent) 3%, var(--bg-elev));
		border-radius: 10px;
		padding: 12px 14px;
	}
	.header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
	}
	.title {
		display: flex;
		align-items: center;
		gap: 6px;
		min-width: 0;
		flex-wrap: wrap;
	}
	.title-icon {
		color: var(--accent);
		flex-shrink: 0;
	}
	.title-text {
		font-size: 12px;
		font-weight: 600;
		color: var(--text);
	}
	.title-meta {
		font-size: 11px;
		color: var(--text-mute);
	}
	.btn-ghost-mini {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		height: 24px;
		padding: 0 8px;
		border-radius: 6px;
		font-size: 11px;
		color: var(--text-mute);
		background: transparent;
		border: none;
		cursor: pointer;
		transition: background 120ms ease, color 120ms ease;
	}
	.btn-ghost-mini:hover {
		background: var(--hover);
		color: var(--text);
	}
	.preview-grid {
		display: grid;
		grid-template-columns: 1fr 24px 1fr;
		gap: 10px;
		align-items: stretch;
		margin-top: 10px;
	}
	.preview-side {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.side-label {
		font-size: 10px;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		font-weight: 600;
		color: var(--text-mute);
		display: flex;
		align-items: center;
		gap: 6px;
	}
	.side-meta {
		font-weight: 400;
		text-transform: none;
		letter-spacing: 0;
		font-size: 10.5px;
	}
	.prose-area {
		width: 100%;
		background: var(--bg);
		border: 1px solid var(--border-strong);
		border-radius: 7px;
		padding: 10px 12px;
		font-family: 'Inter', sans-serif;
		font-size: 12.5px;
		line-height: 1.55;
		color: var(--text);
		resize: none;
		outline: none;
		transition: border-color 120ms ease, box-shadow 120ms ease;
	}
	.prose-area:focus {
		border-color: color-mix(in oklch, var(--accent) 50%, var(--border-strong));
		box-shadow: 0 0 0 3px var(--accent-ring);
	}
	.prose-area:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}
	.preview-arrow {
		align-self: center;
		color: var(--text-mute);
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
		padding-top: 24px;
	}
	.preview-output {
		flex: 1;
		min-height: 96px;
		background: var(--bg);
		border: 1px solid var(--border-strong);
		border-radius: 7px;
		padding: 10px 12px;
		font-size: 12.5px;
		line-height: 1.55;
		color: var(--text);
		overflow-y: auto;
	}
	.preview-output.error {
		border-color: color-mix(in oklch, oklch(0.72 0.18 25) 35%, transparent);
		background: oklch(0.72 0.18 25 / 0.1);
	}
	.placeholder {
		color: var(--text-mute);
		font-style: italic;
	}
	.busy {
		display: flex;
		align-items: center;
		gap: 8px;
		color: var(--text-mute);
		font-size: 12px;
	}
	.spin {
		animation: spin 1s linear infinite;
	}
	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
	.error-body {
		display: flex;
		gap: 8px;
		color: oklch(0.85 0.1 25);
	}
	.error-icon {
		margin-top: 2px;
		flex-shrink: 0;
	}
	.error-text {
		font-size: 12px;
		line-height: 1.4;
	}
	.preview-footer {
		display: flex;
		align-items: center;
		gap: 12px;
		margin-top: 10px;
		padding-top: 10px;
		border-top: 1px solid var(--border);
	}
	.preview-meta {
		font-family: 'JetBrains Mono', ui-monospace, monospace;
		font-size: 10.5px;
		color: var(--text-mute);
	}
	.diff-add {
		background: oklch(0.72 0.16 155 / 0.18);
		color: oklch(0.86 0.1 155);
		border-radius: 3px;
		padding: 0 2px;
	}
	.diff-rem {
		background: oklch(0.72 0.18 25 / 0.16);
		color: oklch(0.82 0.12 25);
		text-decoration: line-through;
		text-decoration-color: oklch(0.72 0.18 25 / 0.6);
		border-radius: 3px;
		padding: 0 2px;
	}
	.highlight-toggle {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		font-size: 11px;
		color: var(--text-mute);
		cursor: pointer;
	}
	.highlight-toggle input {
		accent-color: var(--accent);
	}
	.highlight-toggle input:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}
	.pill-err {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 0 8px;
		height: 22px;
		border-radius: 999px;
		font-size: 11px;
		font-weight: 500;
		background: oklch(0.72 0.18 25 / 0.1);
		color: oklch(0.82 0.14 25);
		border: 1px solid color-mix(in oklch, oklch(0.72 0.18 25) 35%, transparent);
	}
	.pill-err .dot {
		width: 6px;
		height: 6px;
		border-radius: 999px;
		background: oklch(0.72 0.18 25);
	}
</style>
