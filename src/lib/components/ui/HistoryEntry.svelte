<script lang="ts">
	import type { HistoryEntry as HistoryEntryData } from '$lib/ipc/commands';
	import { formatAbsoluteTime, formatDuration, formatRelativeTime } from '$lib/utils/format';
	import IconCopy from './icons/IconCopy.svelte';
	import IconReinsert from './icons/IconReinsert.svelte';
	import IconTrash from './icons/IconTrash.svelte';

	interface Props {
		entry: HistoryEntryData;
		onCopy: (text: string) => void;
		onReinject: (id: number) => void;
		onRetry: (id: number) => void;
		onDelete: (id: number) => void;
	}
	let { entry, onCopy, onReinject, onRetry, onDelete }: Props = $props();

	const isFailed = $derived(entry.status === 'failed');
	const canRetry = $derived(isFailed && entry.failed_wav_path != null);
	const relativeTime = $derived(formatRelativeTime(entry.created_at));
	const absoluteTime = $derived(formatAbsoluteTime(entry.created_at));
	const duration = $derived(formatDuration(entry.duration_ms));
	const snippet = $derived(
		entry.text.length > 180 ? entry.text.slice(0, 177) + '…' : entry.text
	);
	const wordCount = $derived(
		entry.text.trim() ? entry.text.trim().split(/\s+/).length : 0
	);

	/** Colored dot per engine — matches Claude Design palette. */
	const engineDotColor = $derived.by(() => {
		if (entry.engine === 'local') return 'oklch(0.7 0.12 155)';
		if (entry.engine === 'groq') return 'oklch(0.7 0.15 280)';
		return 'var(--text-mute)';
	});

	/** Shorten Windows-style window titles like "foo — Slack" or "inbox - Gmail
	 *  - Mozilla Firefox" so the meta column stays readable. We keep the last
	 *  " — " segment (usually the app name) if the title has one; otherwise
	 *  truncate to 28 chars. */
	const sourceLabel = $derived.by(() => {
		const raw = entry.source_app;
		if (!raw) return null;
		const trimmed = raw.trim();
		if (!trimmed) return null;
		// Window titles often end with " — App Name" or " - App Name"; prefer
		// that as the most recognizable label.
		const match = trimmed.match(/[\-—–]\s*([^\-—–]+)$/);
		const pick = match ? match[1].trim() : trimmed;
		return pick.length > 28 ? pick.slice(0, 27) + '…' : pick;
	});

	/** Friendly engine label — matches what's in history rows of the design. */
	const engineLabel = $derived.by(() => {
		if (entry.engine === 'local') {
			if (!entry.model) return 'On-device';
			const match = entry.model.match(/^ggml-(.+)$/i);
			return match ? `On-device (${match[1]})` : `On-device (${entry.model})`;
		}
		if (entry.engine === 'groq') {
			if (!entry.model) return 'Groq';
			return entry.model
				.split(/[-_]/)
				.map((w) => (w.length > 0 ? w[0].toUpperCase() + w.slice(1) : w))
				.join(' ');
		}
		return entry.engine;
	});
</script>

<div
	class="group bg-hover border-hair relative flex items-start gap-4 border-b px-6 py-4 transition-colors"
	class:border-hair-strong={isFailed}
>
	<!-- Meta column (timestamp + source app) -->
	<div class="w-[108px] shrink-0 pt-0.5">
		<div class="text-base-mute font-mono text-[11px] tabular-nums" title={absoluteTime}>
			{relativeTime}
		</div>
		{#if sourceLabel}
			<div
				class="text-base-mute mt-1 truncate text-[10.5px]"
				title={entry.source_app ?? ''}
			>
				{sourceLabel}
			</div>
		{/if}
	</div>

	<!-- Text + badges -->
	<div class="min-w-0 flex-1">
		{#if isFailed}
			<div class="flex items-center gap-2">
				<span
					class="rounded px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wider"
					style="background: oklch(0.65 0.18 25 / 0.15); color: oklch(0.75 0.14 25)"
				>
					Falló
				</span>
				<span class="text-base-mute text-[12px]">
					{entry.failure_reason ?? 'error desconocido'}
				</span>
			</div>
			{#if entry.text}
				<p
					class="text-base-dim mt-1 text-[13.5px] leading-[1.55]"
					style="display:-webkit-box;-webkit-line-clamp:2;-webkit-box-orient:vertical;overflow:hidden;text-wrap:pretty"
				>{snippet}</p>
			{/if}
		{:else}
			<p
				class="text-base-strong text-[13.5px] leading-[1.55]"
				style="display:-webkit-box;-webkit-line-clamp:2;-webkit-box-orient:vertical;overflow:hidden;text-wrap:pretty"
			>{snippet}</p>
		{/if}

		<div class="mt-2 flex flex-wrap items-center gap-1.5">
			<!-- Engine badge with colored dot -->
			<span
				class="border-hair bg-elev text-base-dim inline-flex items-center gap-1.5 rounded border px-1.5 py-0.5 text-[10.5px] font-medium"
			>
				<span
					class="h-1.5 w-1.5 rounded-full"
					style="background: {engineDotColor}"
					aria-hidden="true"
				></span>
				{engineLabel}
			</span>
			<!-- Language mono -->
			<span
				class="border-hair bg-elev text-base-dim inline-flex items-center rounded border px-1.5 py-0.5 font-mono text-[10.5px] uppercase tracking-wider"
			>
				{entry.language}
			</span>
			<!-- Duration mono -->
			{#if duration}
				<span
					class="border-hair bg-elev text-base-dim inline-flex items-center rounded border px-1.5 py-0.5 font-mono text-[10.5px]"
				>
					{duration}
				</span>
			{/if}
			<!-- Word count (derived) -->
			{#if wordCount > 0}
				<span class="text-base-mute font-mono text-[10.5px]">·</span>
				<span class="text-base-mute font-mono text-[10.5px]">{wordCount} palabras</span>
			{/if}
		</div>
	</div>

	<!-- Hover actions -->
	<div
		class="bg-panel border-hair absolute top-2 right-2 flex items-center gap-1 rounded-md border p-0.5 opacity-0 transition-opacity group-hover:opacity-100"
	>
		{#if canRetry}
			<button
				class="bg-hover rounded px-2 py-1 text-[11px] font-medium transition-colors"
				title="Re-transcribir el audio preservado"
				style="color: var(--accent)"
				onclick={() => onRetry(entry.id)}
			>
				Reintentar
			</button>
		{/if}
		<button
			class="bg-hover text-base-dim hover:text-base-strong rounded p-1.5 transition-colors"
			title="Copiar"
			onclick={() => onCopy(entry.text)}
			disabled={!entry.text}
		>
			<IconCopy size={13} />
		</button>
		<button
			class="bg-hover text-base-dim hover:text-base-strong rounded p-1.5 transition-colors"
			title="Re-insertar"
			onclick={() => onReinject(entry.id)}
			disabled={!entry.text}
		>
			<IconReinsert size={13} />
		</button>
		<button
			class="bg-hover rounded p-1.5 transition-colors hover:text-[oklch(0.72_0.18_25)]"
			title="Borrar"
			style="color: var(--text-dim)"
			onclick={() => onDelete(entry.id)}
		>
			<IconTrash size={13} />
		</button>
	</div>
</div>
