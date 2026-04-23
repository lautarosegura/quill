<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { Button, PasswordInput, ProgressCard } from '$lib/components/ui';
	import * as api from '$lib/ipc/commands';
	import type {
		ModelDownloadComplete,
		ModelDownloadError,
		ModelDownloadProgress
	} from '$lib/ipc/commands';
	import { wizard } from '$lib/stores/wizard.svelte';

	let downloadProgress = $state<{ downloaded: number; total: number } | null>(null);
	let downloadError = $state<string | null>(null);

	// ETA computation via rolling samples: keep last N timestamped progress
	// points and derive bytes/sec from the window, so a single stalled update
	// doesn't spike the estimate.
	type Sample = { t: number; downloaded: number };
	let etaSamples: Sample[] = [];
	let etaText = $state<string | null>(null);

	let testState = $state<'idle' | 'testing' | 'success' | 'error'>('idle');
	let testMessage = $state<string>('');

	const unlistens: UnlistenFn[] = [];

	const choice = $derived(wizard.draft.engineChoice);
	const needsLocal = $derived(choice === 'local' || choice === 'both');
	const needsGroq = $derived(choice === 'groq' || choice === 'both');
	const modelToDownload = $derived(wizard.draft.localModel);

	async function startDownload() {
		downloadError = null;
		downloadProgress = { downloaded: 0, total: 1 };
		try {
			await api.downloadModel(modelToDownload);
		} catch (e) {
			console.error('downloadModel:', e);
		}
	}

	async function testKey() {
		if (!wizard.draft.groqKey.trim()) return;
		testState = 'testing';
		testMessage = '';
		try {
			const result = await api.testGroqKey(wizard.draft.groqKey.trim());
			testState = result.valid ? 'success' : 'error';
			testMessage = result.message;
			wizard.patch('groqKeyTested', result.valid);
		} catch (e) {
			testState = 'error';
			testMessage = e instanceof Error ? e.message : String(e);
		}
	}

	onMount(async () => {
		// Check if local model is already downloaded (from a previous attempt).
		if (needsLocal) {
			try {
				const models = await api.listKnownModels();
				const target = models.find((m) => m.name === modelToDownload);
				if (target?.installed) {
					wizard.patch('localModelDownloaded', true);
				}
			} catch {}
		}

		unlistens.push(
			await listen<ModelDownloadProgress>('model_download_progress', (event) => {
				if (event.payload.name === modelToDownload) {
					downloadProgress = {
						downloaded: event.payload.downloaded,
						total: event.payload.total
					};
					// Update rolling sample window (keep last ~10s).
					const now = performance.now();
					etaSamples.push({ t: now, downloaded: event.payload.downloaded });
					etaSamples = etaSamples.filter((s) => now - s.t < 10_000);
					etaText = computeEta(event.payload.downloaded, event.payload.total);
				}
			})
		);
		unlistens.push(
			await listen<ModelDownloadComplete>('model_download_complete', (event) => {
				if (event.payload.name === modelToDownload) {
					wizard.patch('localModelDownloaded', true);
					downloadProgress = null;
				}
			})
		);
		unlistens.push(
			await listen<ModelDownloadError>('model_download_error', (event) => {
				if (event.payload.name === modelToDownload) {
					downloadError = event.payload.message;
					downloadProgress = null;
				}
			})
		);
	});

	onDestroy(() => {
		unlistens.forEach((fn) => fn());
	});

	function computeEta(downloaded: number, total: number): string | null {
		if (etaSamples.length < 2 || total <= 0 || downloaded >= total) return null;
		const first = etaSamples[0];
		const last = etaSamples[etaSamples.length - 1];
		const deltaBytes = last.downloaded - first.downloaded;
		const deltaSec = (last.t - first.t) / 1000;
		if (deltaSec <= 0 || deltaBytes <= 0) return null;
		const bytesPerSec = deltaBytes / deltaSec;
		const remainingSec = (total - downloaded) / bytesPerSec;
		if (!Number.isFinite(remainingSec) || remainingSec < 1) return null;
		if (remainingSec < 60) return `${Math.round(remainingSec)}s restante`;
		const min = Math.round(remainingSec / 60);
		return `${min} min restante${min === 1 ? '' : 's'}`;
	}
</script>

<div class="flex flex-col gap-4">
	<div>
		<h2 class="text-base-strong text-[18px] font-semibold tracking-tight">Configurá tu motor</h2>
		<p class="text-base-dim mt-1 text-[13px]">
			{choice === 'both'
				? 'Armamos ambos motores. Podés elegir uno como default después en Ajustes.'
				: choice === 'groq'
					? 'Pegá tu API key de Groq y verificala.'
					: 'Vamos a bajar el modelo recomendado.'}
		</p>
	</div>

	{#if needsLocal}
		<div class="flex flex-col gap-2">
			{#if wizard.draft.localModelDownloaded}
				<ProgressCard
					title={modelToDownload}
					meta="Whisper · modelo local"
					value={1}
					total={1}
				/>
			{:else if downloadProgress}
				<ProgressCard
					title={modelToDownload}
					meta="Whisper · multilingual · desde Hugging Face"
					value={downloadProgress.downloaded}
					total={downloadProgress.total}
					eta={etaText ?? undefined}
					bytes
				/>
			{:else}
				<div class="border-hair bg-elev flex items-center justify-between rounded-lg border p-3.5">
					<div class="min-w-0 flex-1">
						<div class="text-base-strong text-[12.5px] font-semibold">{modelToDownload}</div>
						<div class="text-base-mute mt-0.5 truncate font-mono text-[11px]">
							Whisper · multilingual · descarga en segundo plano
						</div>
					</div>
					<Button variant="primary" size="sm" onclick={startDownload}>Descargar</Button>
				</div>
			{/if}

			{#if downloadError}
				<p class="text-[11.5px]" style="color: oklch(0.72 0.18 25);">
					Error: {downloadError}. Podés reintentar o saltear este paso y bajar después desde
					Modelos.
				</p>
			{/if}
		</div>
	{/if}

	{#if needsGroq}
		<div class="border-hair bg-panel rounded-lg border p-4">
			<div class="flex items-center justify-between">
				<div class="text-base-strong text-[13px] font-semibold">API Key de Groq</div>
				<button
					type="button"
					class="text-base-dim hover:text-base-strong text-[11.5px] underline-offset-2 hover:underline"
					onclick={() =>
						import('@tauri-apps/plugin-shell').then((m) =>
							m.open('https://groq.com/').catch(console.error)
						)}
				>
					Obtener una clave ↗
				</button>
			</div>
			<div class="mt-3">
				<PasswordInput
					value={wizard.draft.groqKey}
					onChange={(v) => {
						wizard.patch('groqKey', v);
						if (wizard.draft.groqKeyTested) {
							wizard.patch('groqKeyTested', false);
							testState = 'idle';
							testMessage = '';
						}
					}}
					placeholder="gsk_..."
				/>
			</div>
			<div class="mt-3 flex items-center gap-2">
				<Button
					variant="secondary"
					size="sm"
					onclick={testKey}
					disabled={!wizard.draft.groqKey.trim() || testState === 'testing'}
				>
					{testState === 'testing' ? 'Probando…' : 'Probar clave'}
				</Button>
				{#if testMessage}
					<span
						class="text-[12px] font-medium"
						style="color: {testState === 'success'
							? 'oklch(0.75 0.16 155)'
							: 'oklch(0.72 0.18 25)'};"
					>
						{testMessage}
					</span>
				{/if}
			</div>
			<p class="text-base-mute mt-3 text-[11px]">
				Tu clave se guarda en el keychain del sistema cuando termines el wizard.
			</p>
		</div>
	{/if}
</div>
