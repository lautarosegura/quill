<script lang="ts">
	import { Button, PasswordInput } from '$lib/components/ui';
	import * as api from '$lib/ipc/commands';
	import type { LlmProvider } from '$lib/types';

	interface Props {
		provider: LlmProvider;
		/** Reactively-tracked masked key for this provider, owned by parent so
		 *  parent can rebuild the segmented "key configured" dots. */
		masked: string | null;
		/** Bubble up state changes so the parent can re-fetch / refresh. */
		onMaskedChange: (masked: string | null) => void;
	}

	let { provider, masked, onMaskedChange }: Props = $props();

	const providerLabels: Record<LlmProvider, string> = {
		groq: 'Groq',
		anthropic: 'Anthropic',
		openai: 'OpenAI'
	};

	const providerPlaceholders: Record<LlmProvider, string> = {
		groq: 'gsk_…',
		anthropic: 'sk-ant-…',
		openai: 'sk-…'
	};

	const providerKeyUrl: Record<LlmProvider, string> = {
		groq: 'https://console.groq.com/keys',
		anthropic: 'https://console.anthropic.com/settings/keys',
		openai: 'https://platform.openai.com/api-keys'
	};

	let input = $state('');
	let editing = $state(false); // true when typing a new key over an existing one
	let testState = $state<'idle' | 'testing' | 'success' | 'error'>('idle');
	let testMessage = $state<string>('');

	// Reset transient state when the active provider changes — switching tabs
	// shouldn't carry an old test result over.
	$effect(() => {
		// re-run on provider change
		void provider;
		input = '';
		editing = false;
		testState = 'idle';
		testMessage = '';
	});

	const showInput = $derived(!masked || editing);

	async function testKey() {
		if (!input.trim()) return;
		testState = 'testing';
		testMessage = '';
		try {
			const r = await api.testLlmPolishKey(provider, input.trim());
			testState = r.valid ? 'success' : 'error';
			testMessage = r.message;
		} catch (e: unknown) {
			testState = 'error';
			testMessage = e instanceof Error ? e.message : String(e);
		}
	}

	async function saveKey() {
		if (!input.trim()) return;
		try {
			await api.setLlmPolishKey(provider, input.trim());
			const fresh = await api.getLlmPolishKeyMasked(provider);
			onMaskedChange(fresh);
			input = '';
			editing = false;
			testState = 'success';
			testMessage = 'Clave guardada';
		} catch (e: unknown) {
			testState = 'error';
			testMessage = e instanceof Error ? e.message : String(e);
		}
	}

	async function deleteKey() {
		try {
			await api.deleteLlmPolishKey(provider);
			onMaskedChange(null);
			input = '';
			editing = false;
			testState = 'idle';
			testMessage = '';
		} catch (e: unknown) {
			testState = 'error';
			testMessage = e instanceof Error ? e.message : String(e);
		}
	}

	function startEditing() {
		editing = true;
		input = '';
		testState = 'idle';
		testMessage = '';
	}

	function cancelEditing() {
		editing = false;
		input = '';
		testState = 'idle';
		testMessage = '';
	}

	function openProviderConsole() {
		// Lazy import to avoid pulling Tauri shell on initial component eval.
		import('@tauri-apps/plugin-shell').then(({ open }) => {
			open(providerKeyUrl[provider]).catch(console.error);
		});
	}
</script>

<div class="key-panel" class:empty={!masked && !editing} class:has-key={!!masked}>
	<div class="header">
		<span class="label">API key · {providerLabels[provider]}</span>
	</div>

	{#if showInput}
		<!-- Empty / editing state -->
		<div class="row">
			<PasswordInput
				value={input}
				onChange={(v) => (input = v)}
				placeholder={providerPlaceholders[provider]}
			/>
			<Button
				variant="secondary"
				size="sm"
				onclick={testKey}
				disabled={!input.trim() || testState === 'testing'}
			>
				{testState === 'testing' ? 'Probando…' : 'Probar'}
			</Button>
			<Button variant="primary" size="sm" onclick={saveKey} disabled={!input.trim()}>
				Guardar
			</Button>
			{#if masked && editing}
				<Button variant="ghost" size="sm" onclick={cancelEditing}>Cancelar</Button>
			{/if}
		</div>

		{#if !masked}
			<button type="button" class="key-link" onclick={openProviderConsole}>
				Conseguí una clave en {new URL(providerKeyUrl[provider]).host} ↗
			</button>
		{/if}
	{:else}
		<!-- Saved state -->
		<div class="row saved-row">
			<span class="pill-ok"><span class="dot"></span>Configurada</span>
			<span class="masked">{masked}</span>
			<div class="ml-auto flex items-center gap-2">
				<Button variant="secondary" size="sm" onclick={startEditing}>Cambiar clave</Button>
				<Button variant="ghost" size="sm" onclick={deleteKey}>Borrar</Button>
			</div>
		</div>
	{/if}

	{#if testMessage}
		{#if testState === 'success'}
			<span class="pill-ok self-start mt-2"><span class="dot"></span>{testMessage}</span>
		{:else if testState === 'error'}
			<span class="pill-err self-start mt-2"><span class="dot"></span>{testMessage}</span>
		{:else}
			<span class="text-base-mute text-[12px] mt-2">{testMessage}</span>
		{/if}
	{/if}
</div>

<style>
	.key-panel {
		display: flex;
		flex-direction: column;
		gap: 10px;
		border: 1px solid var(--border);
		background: var(--bg-elev);
		border-radius: 8px;
		padding: 12px 14px;
	}
	.key-panel.empty {
		background: color-mix(in oklch, var(--accent) 5%, var(--bg-elev));
		border-color: color-mix(in oklch, var(--accent) 25%, var(--border));
	}
	.header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}
	.label {
		font-size: 10px;
		letter-spacing: 0.1em;
		text-transform: uppercase;
		font-weight: 600;
		color: var(--text-mute);
	}
	.key-panel.empty .label {
		color: var(--accent);
	}
	.row {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-wrap: wrap;
	}
	.saved-row {
		flex-wrap: wrap;
	}
	.masked {
		font-family: 'JetBrains Mono', ui-monospace, monospace;
		font-size: 12px;
		color: var(--text);
		font-weight: 500;
	}
	.key-link {
		align-self: flex-start;
		font-size: 11px;
		font-weight: 500;
		color: var(--accent);
		background: none;
		border: none;
		padding: 0;
		cursor: pointer;
	}
	.key-link:hover {
		text-decoration: underline;
	}
	.pill-ok {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 0 8px;
		height: 22px;
		border-radius: 999px;
		font-size: 11px;
		font-weight: 500;
		background: oklch(0.75 0.16 155 / 0.12);
		color: oklch(0.82 0.13 155);
		border: 1px solid color-mix(in oklch, oklch(0.75 0.16 155) 35%, transparent);
	}
	.pill-ok .dot {
		width: 6px;
		height: 6px;
		border-radius: 999px;
		background: oklch(0.75 0.16 155);
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
