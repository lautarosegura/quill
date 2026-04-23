<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		size?: 'inline' | 'big';
		children?: Snippet;
	}
	let { size = 'inline', children }: Props = $props();
</script>

{#if size === 'big'}
	<span class="kbd-big">{@render children?.()}</span>
{:else}
	<span class="kbd-inline">{@render children?.()}</span>
{/if}
