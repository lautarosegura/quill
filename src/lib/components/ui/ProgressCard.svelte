<script lang="ts">
	import IconCheck from './icons/IconCheck.svelte';
	import ProgressBar from './ProgressBar.svelte';
	import { formatBytes } from '$lib/utils/format';

	interface Props {
		title: string;
		meta?: string;
		/** Current value in bytes (for downloads) or units. */
		value: number;
		/** Total value. 0 means "indeterminate" — we only show percent if total > 0. */
		total: number;
		/** Optional: "2 min remaining" type hint. */
		eta?: string;
		/** Optional: show a size readout instead of raw percent (e.g. downloads). */
		bytes?: boolean;
	}
	let { title, meta, value, total, eta, bytes = false }: Props = $props();

	const pct = $derived(total > 0 ? Math.round((value / total) * 100) : 0);
	const done = $derived(total > 0 && value >= total);
</script>

<div class="border-hair bg-elev rounded-lg border p-3.5">
	<div class="flex items-center justify-between gap-3">
		<div class="min-w-0 flex-1">
			<div class="text-base-strong truncate text-[12.5px] font-semibold">{title}</div>
			{#if meta}
				<div class="text-base-mute mt-0.5 truncate font-mono text-[11px]">{meta}</div>
			{/if}
		</div>
		{#if done}
			<span
				class="inline-flex h-6 shrink-0 items-center gap-1 rounded px-2 text-[10.5px] font-semibold"
				style="background: oklch(0.72 0.16 155 / 0.14); color: oklch(0.75 0.16 155);"
			>
				<IconCheck size={10} /> Listo
			</span>
		{:else}
			<span class="text-base-dim shrink-0 font-mono text-[11px] tabular-nums">
				{#if bytes && total > 0}
					{formatBytes(value)} / {formatBytes(total)}
				{:else}
					{pct}%
				{/if}
			</span>
		{/if}
	</div>

	{#if !done}
		<div class="mt-3 flex items-center gap-3">
			<div class="flex-1">
				<ProgressBar {value} max={total} height={4} shimmer={true} />
			</div>
			{#if eta}
				<span class="text-base-mute shrink-0 font-mono text-[10.5px]">{eta}</span>
			{/if}
		</div>
	{/if}
</div>
