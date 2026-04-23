<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		size?: number;
		stroke?: number;
		class?: string;
		children?: Snippet;
	}
	let { size = 16, stroke = 1.5, class: className = '', children }: Props = $props();
</script>

<svg
	width={size}
	height={size}
	viewBox="0 0 24 24"
	fill="none"
	stroke="currentColor"
	stroke-width={stroke}
	stroke-linecap="round"
	stroke-linejoin="round"
	class={className}
>
	{@render children?.()}
</svg>
