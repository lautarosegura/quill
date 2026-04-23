<script lang="ts">
	interface Props {
		color?: string;
	}
	let { color = 'oklch(0.85 0.14 80)' }: Props = $props();
</script>

<div class="flex items-center gap-1">
	{#each [0, 1, 2] as i}
		<span
			class="h-1.5 w-1.5 rounded-full"
			style="background: {color}; animation: dotPulse 1.2s ease-in-out {i * 0.18}s infinite"
		></span>
	{/each}
</div>
