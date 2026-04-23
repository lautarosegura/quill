<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		text: string;
		shortcut?: string;
		children?: Snippet;
	}
	let { text, shortcut, children }: Props = $props();
</script>

<span class="tip">
	{@render children?.()}
	<span class="tip-body">
		{text}
		{#if shortcut}
			<span class="ml-2 font-mono text-[10px] opacity-60">{shortcut}</span>
		{/if}
	</span>
</span>
