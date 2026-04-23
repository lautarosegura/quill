<script lang="ts">
	interface Props {
		value: number;
		max: number;
		height?: number;
		/** Show a shimmering highlight band over the filled portion while active.
		 *  Auto-disabled when `value >= max` so finished bars don't jitter. */
		shimmer?: boolean;
	}
	let { value, max, height = 4, shimmer = false }: Props = $props();

	const pct = $derived(max > 0 ? Math.min(100, Math.max(0, (value / max) * 100)) : 0);
	const showShimmer = $derived(shimmer && pct > 0 && pct < 100);
</script>

<div
	class="overflow-hidden rounded-full"
	style="height: {height}px; background: var(--border-strong);"
>
	<div
		class="h-full rounded-full transition-all"
		class:progress-shimmer={showShimmer}
		style="width: {pct}%; background: var(--accent);"
	></div>
</div>
