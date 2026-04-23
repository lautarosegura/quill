<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { CloudModelCard, ModelCard, Modal, Toast } from '$lib/components/ui';
	import * as api from '$lib/ipc/commands';
	import type {
		GroqModelsResult,
		HardwareProfile,
		ModelDownloadComplete,
		ModelDownloadError,
		ModelDownloadProgress,
		ModelEntry
	} from '$lib/ipc/commands';
	import { config } from '$lib/stores/config.svelte';

	let hardware = $state<HardwareProfile | null>(null);
	let models = $state<ModelEntry[]>([]);
	let groqModels = $state<GroqModelsResult | null>(null);
	let groqLoading = $state(false);
	let groqKeyConfigured = $state(false);
	let downloading = $state<Record<string, { downloaded: number; total: number }>>({});
	let deleteTarget = $state<ModelEntry | null>(null);
	let toast = $state<{ kind: 'success' | 'info' | 'error'; message: string } | null>(null);
	let toastTimer: ReturnType<typeof setTimeout> | null = null;

	const unlistens: UnlistenFn[] = [];

	async function refresh() {
		try {
			models = await api.listKnownModels();
		} catch (e) {
			flashToast('error', `Error listando modelos: ${String(e)}`);
		}
	}

	async function refreshGroq() {
		groqLoading = true;
		try {
			const masked = await api.getGroqKeyMasked();
			groqKeyConfigured = masked != null;
			groqModels = await api.listGroqModels();
		} catch (e) {
			flashToast('error', `Error listando modelos Groq: ${String(e)}`);
			groqModels = null;
		} finally {
			groqLoading = false;
		}
	}

	async function onSetGroqActive(name: string) {
		try {
			await api.setGroqModel(name);
			await config.load();
			flashToast('info', `Modelo Groq activo: ${name}`);
		} catch (e) {
			flashToast('error', String(e));
		}
	}

	function flashToast(kind: 'success' | 'info' | 'error', message: string) {
		toast = { kind, message };
		if (toastTimer) clearTimeout(toastTimer);
		toastTimer = setTimeout(() => (toast = null), 3000);
	}

	onMount(async () => {
		try {
			hardware = await api.detectHardware();
		} catch {
			hardware = null;
		}
		await refresh();
		if (!config.value) await config.load();
		await refreshGroq();

		unlistens.push(
			await listen<ModelDownloadProgress>('model_download_progress', (event) => {
				const { name, downloaded, total } = event.payload;
				downloading = { ...downloading, [name]: { downloaded, total } };
			})
		);
		unlistens.push(
			await listen<ModelDownloadComplete>('model_download_complete', async (event) => {
				const { name } = event.payload;
				const next = { ...downloading };
				delete next[name];
				downloading = next;
				flashToast('success', `${name} descargado`);
				await refresh();
			})
		);
		unlistens.push(
			await listen<ModelDownloadError>('model_download_error', (event) => {
				const { name, message } = event.payload;
				const next = { ...downloading };
				delete next[name];
				downloading = next;
				flashToast('error', `Error descargando ${name}: ${message}`);
			})
		);
	});

	onDestroy(() => {
		unlistens.forEach((fn) => fn());
	});

	async function onDownload(name: string) {
		downloading = { ...downloading, [name]: { downloaded: 0, total: 1 } };
		try {
			await api.downloadModel(name);
		} catch (e) {
			// The backend also emits an error event; swallow the thrown error to
			// avoid double-toasting.
			console.error('downloadModel threw:', e);
		}
	}

	async function onSetActive(name: string) {
		if (!config.value) return;
		try {
			await config.set('local_model_name', name);
			flashToast('info', `Modelo activo: ${name}`);
		} catch (e) {
			flashToast('error', String(e));
		}
	}

	async function confirmDelete() {
		if (!deleteTarget) return;
		const name = deleteTarget.name;
		deleteTarget = null;
		try {
			await api.deleteModel(name);
			await refresh();
			flashToast('info', `${name} borrado`);
		} catch (e) {
			flashToast('error', String(e));
		}
	}

	function onRequestDelete(model: ModelEntry) {
		deleteTarget = model;
	}

	const currentActive = $derived(config.value?.local_model_name ?? '');
	const recommendedName = $derived(hardware?.recommended_model ?? '');
	const currentGroqModel = $derived(config.value?.groq_model ?? '');
</script>

<div class="mx-auto max-w-[860px] p-8">
	<h1 class="text-base-strong text-[22px] font-semibold tracking-tight">Modelos</h1>
	<p class="text-base-dim mt-1 text-sm">
		Modelos Whisper locales. Bajalos desde acá o desde el wizard de primer uso.
	</p>

	<!-- Hardware banner -->
	{#if hardware}
		<div
			class="mt-6 flex items-center gap-3 rounded-lg border p-4"
			style="border-color: color-mix(in oklch, var(--accent) 25%, transparent); background: var(--accent-soft);"
		>
			<span class="text-2xl">💻</span>
			<div class="min-w-0 flex-1">
				<div class="text-base-strong text-[13px] font-semibold">
					Detectamos: {hardware.os} / {hardware.arch}
					{#if hardware.apple_silicon}· Apple Silicon{/if}
					· {hardware.ram_gb} GB RAM · {hardware.cpu_cores} cores
				</div>
				<div class="text-base-dim mt-0.5 text-[12px]">
					Recomendamos <span class="accent-text font-mono font-semibold"
						>{hardware.recommended_model}</span
					> — {hardware.recommended_rationale}
				</div>
			</div>
		</div>
	{/if}

	<!-- ── Local (on-device) ── -->
	<div class="mt-8 mb-3 flex items-baseline gap-2">
		<span class="text-base-mute text-[10.5px] font-semibold tracking-[0.08em] uppercase">
			Local (on-device)
		</span>
		<span class="text-base-mute font-mono text-[10.5px]">{models.length}</span>
	</div>

	<div class="flex flex-col gap-3">
		{#each models as model (model.name)}
			<ModelCard
				{model}
				isActive={model.name === currentActive}
				isRecommended={model.name === recommendedName}
				downloadProgress={downloading[model.name] ?? null}
				onDownload={() => onDownload(model.name)}
				onSetActive={() => onSetActive(model.name)}
				onDelete={() => onRequestDelete(model)}
			/>
		{/each}
	</div>

	<div class="text-base-mute mt-3 text-center text-[11px]">
		Descargados desde
		<a
			href="https://huggingface.co/ggerganov/whisper.cpp"
			target="_blank"
			rel="noopener"
			class="underline-offset-2 hover:underline">huggingface.co/ggerganov/whisper.cpp</a
		>
	</div>

	<!-- ── Groq Cloud ── -->
	<div class="mt-10 mb-3 flex items-baseline gap-2">
		<span class="text-base-mute text-[10.5px] font-semibold tracking-[0.08em] uppercase">
			Groq Cloud
		</span>
		{#if groqModels && groqModels.models.length > 0}
			<span class="text-base-mute font-mono text-[10.5px]">{groqModels.models.length}</span>
		{/if}
		{#if groqLoading}
			<span class="text-base-mute text-[10.5px]">cargando…</span>
		{/if}
	</div>

	{#if !groqKeyConfigured}
		<div class="border-hair bg-panel rounded-lg border p-6 text-center">
			<p class="text-base-dim text-[13px]">
				Configurá tu clave API de Groq en <a
					href="/main/settings"
					class="accent-text underline-offset-2 hover:underline">Ajustes → Motor → Groq Cloud</a
				>
				para ver los modelos disponibles.
			</p>
		</div>
	{:else if groqModels}
		{#if !groqModels.live_check_succeeded && groqModels.error}
			<div
				class="mb-3 flex items-start gap-3 rounded-md border p-3"
				style="border-color: oklch(0.78 0.14 75 / 0.35); background: oklch(0.78 0.14 75 / 0.08);"
			>
				<span class="text-[16px]">⚠️</span>
				<p class="text-[12px]" style="color: oklch(0.82 0.1 75);">
					{groqModels.error}
				</p>
			</div>
		{/if}

		<div class="flex flex-col gap-3">
			{#each groqModels.models as model (model.name)}
				<CloudModelCard
					{model}
					isActive={model.name === currentGroqModel}
					onSetActive={() => onSetGroqActive(model.name)}
				/>
			{/each}
		</div>
	{/if}
</div>

{#if deleteTarget}
	<Modal
		title="¿Borrar {deleteTarget.display_name}?"
		description={`Se van a liberar ${formatSize(deleteTarget.size_bytes)}. Podés volver a descargar el modelo cuando quieras.${
			deleteTarget.name === currentActive
				? ' ⚠️ Es el modelo activo — vas a tener que elegir otro después de borrar.'
				: ''
		}`}
		confirmLabel="Borrar"
		cancelLabel="Cancelar"
		destructive
		onConfirm={confirmDelete}
		onCancel={() => (deleteTarget = null)}
	/>
{/if}

{#if toast}
	<div class="fixed right-6 bottom-6 z-40">
		<Toast kind={toast.kind} message={toast.message} />
	</div>
{/if}

<script lang="ts" module>
	function formatSize(bytes: number): string {
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
		if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(0)} MB`;
		return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
	}
</script>
