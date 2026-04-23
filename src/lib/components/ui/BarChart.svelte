<script lang="ts">
	interface Props {
		data: Array<{ label: string; value: number; tooltip?: string }>;
		height?: number;
		showLabels?: boolean;
	}
	let { data, height = 80, showLabels = true }: Props = $props();

	const maxValue = $derived(Math.max(1, ...data.map((d) => d.value)));
</script>

<div class="flex flex-col gap-1.5">
	<div class="flex items-end gap-[2px]" style="height: {height}px;">
		{#each data as d}
			{@const pct = (d.value / maxValue) * 100}
			<div
				class="flex flex-1 flex-col items-stretch justify-end"
				title={d.tooltip ?? `${d.label}: ${d.value}`}
			>
				<div
					class="rounded-t-sm transition-all"
					style="height: {pct}%; min-height: {d.value > 0 ? 2 : 0}px; background: {d.value > 0
						? 'var(--accent)'
						: 'var(--border-strong)'};"
				></div>
			</div>
		{/each}
	</div>

	{#if showLabels}
		<div class="text-base-mute flex justify-between font-mono text-[9.5px]">
			{#each data as d, i}
				{#if i === 0 || i === data.length - 1 || i === Math.floor(data.length / 2)}
					<span>{d.label}</span>
				{/if}
			{/each}
		</div>
	{/if}
</div>
