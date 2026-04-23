<script lang="ts">
	interface Props {
		active?: boolean;
		color?: string;
	}
	let { active = true, color = 'white' }: Props = $props();

	const bars = [
		{ h1: 8, h2: 18, d: '0s' },
		{ h1: 14, h2: 22, d: '0.1s' },
		{ h1: 6, h2: 26, d: '0.2s' },
		{ h1: 16, h2: 20, d: '0.3s' },
		{ h1: 10, h2: 14, d: '0.15s' }
	];
</script>

<div class="flex h-6 items-center gap-[3px]">
	{#each bars as b}
		<span
			class="w-[2.5px] rounded-full"
			style="
				background: {color};
				height: {b.h2}px;
				animation: {active ? `wave 0.9s ease-in-out ${b.d} infinite alternate` : 'none'};
				transform-origin: center;
				--h1: {b.h1}px;
				--h2: {b.h2}px;
			"
		></span>
	{/each}
</div>
