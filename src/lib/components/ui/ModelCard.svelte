<script lang="ts">
	import Button from './Button.svelte';
	import ProgressBar from './ProgressBar.svelte';
	import { formatBytes } from '$lib/utils/format';

	interface Props {
		model: {
			name: string;
			display_name: string;
			size_bytes: number;
			description: string;
			installed: boolean;
			installed_size_bytes: number | null;
		};
		isActive: boolean;
		isRecommended: boolean;
		/** Current download progress for this model, if any. */
		downloadProgress?: { downloaded: number; total: number } | null;
		onDownload: () => void;
		onDelete: () => void;
		onSetActive: () => void;
	}
	let {
		model,
		isActive,
		isRecommended,
		downloadProgress,
		onDownload,
		onDelete,
		onSetActive
	}: Props = $props();

	const isDownloading = $derived(downloadProgress != null);
	const pctLabel = $derived.by(() => {
		if (!downloadProgress) return '';
		const pct = (downloadProgress.downloaded / downloadProgress.total) * 100;
		return `${pct.toFixed(0)}%`;
	});

	/** Visual size indicator — circle diameter scales with model size.
	 *  Capped at 28px to keep layout stable across the 40x-size range. */
	const sizeCircleDiameter = $derived.by(() => {
		const minPx = 10;
		const maxPx = 28;
		const maxSize = 3_100_000_000; // ~large-v3
		const ratio = Math.min(1, model.size_bytes / maxSize);
		return minPx + Math.round((maxPx - minPx) * Math.sqrt(ratio));
	});
</script>

<div
	class="border-hair bg-panel flex items-start gap-4 rounded-lg border p-4 transition-colors"
	class:ring-accent={isActive}
>
	<!-- Size visualization -->
	<div class="flex h-10 w-10 shrink-0 items-center justify-center">
		<div
			class="rounded-full"
			style="width: {sizeCircleDiameter}px; height: {sizeCircleDiameter}px; background: var(--accent-soft); border: 1px solid var(--accent);"
		></div>
	</div>

	<!-- Info -->
	<div class="min-w-0 flex-1">
		<div class="flex items-center gap-2">
			<span class="text-base-strong text-[14px] font-semibold">{model.display_name}</span>
			<span class="text-base-mute font-mono text-[11px]">· {formatBytes(model.size_bytes)}</span>
			{#if isRecommended}
				<span
					class="rounded px-1.5 py-0.5 text-[10px] font-semibold tracking-wider uppercase"
					style="background: var(--accent-soft); color: var(--accent);"
				>
					Recomendado
				</span>
			{/if}
			{#if isActive}
				<span
					class="rounded px-1.5 py-0.5 text-[10px] font-semibold tracking-wider uppercase"
					style="background: oklch(0.72 0.16 155 / 0.15); color: oklch(0.75 0.14 155);"
				>
					Activo
				</span>
			{/if}
		</div>
		<p class="text-base-dim mt-1 text-[12.5px] leading-relaxed">{model.description}</p>

		{#if isDownloading && downloadProgress}
			<div class="mt-3">
				<ProgressBar value={downloadProgress.downloaded} max={downloadProgress.total} />
				<div class="text-base-mute mt-1.5 flex items-center justify-between font-mono text-[10.5px]">
					<span>
						{formatBytes(downloadProgress.downloaded)} / {formatBytes(downloadProgress.total)}
					</span>
					<span>{pctLabel}</span>
				</div>
			</div>
		{/if}
	</div>

	<!-- Actions -->
	<div class="flex shrink-0 flex-col items-end gap-1.5">
		{#if isDownloading}
			<span class="text-base-dim text-[11.5px] font-medium">Descargando…</span>
		{:else if !model.installed}
			<Button variant="primary" size="sm" onclick={onDownload}>Descargar</Button>
		{:else if isActive}
			<Button variant="ghost" size="sm" onclick={onDelete}>Borrar</Button>
		{:else}
			<Button variant="secondary" size="sm" onclick={onSetActive}>Usar</Button>
			<Button variant="ghost" size="sm" onclick={onDelete}>Borrar</Button>
		{/if}
	</div>
</div>
