<script lang="ts">
	interface Props {
		/** Current RMS level, 0.0–1.0. */
		level: number;
		/** Total number of bars. Design uses 24. */
		bars?: number;
	}
	let { level, bars = 24 }: Props = $props();

	// Clamp for safety — the Rust side also clamps but JS state can lag.
	const value = $derived(Math.max(0, Math.min(1, level)));
</script>

<div
	class="border-hair bg-elev flex h-10 items-center gap-[3px] overflow-hidden rounded-md border px-3"
>
	{#each Array.from({ length: bars }) as _, i}
		{@const threshold = (i + 1) / bars}
		{@const on = value >= threshold}
		{@const color =
			threshold > 0.85
				? 'oklch(0.68 0.18 25)'
				: threshold > 0.7
					? 'oklch(0.78 0.16 85)'
					: 'var(--accent)'}
		<span
			class="flex-1 rounded-[1px] transition-all"
			style:height="{6 + i * 0.8}px"
			style:background={on ? color : 'var(--border-strong)'}
			style:opacity={on ? '1' : '0.45'}
		></span>
	{/each}
</div>
