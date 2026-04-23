<script lang="ts">
	import type { Component } from 'svelte';

	interface Props {
		Icon: Component<{ size?: number; stroke?: number }>;
		label: string;
		active: boolean;
		count?: number;
		onClick: () => void;
	}
	let { Icon, label, active, count, onClick }: Props = $props();
</script>

<button
	onclick={onClick}
	class="group flex w-full items-center gap-2.5 rounded-md px-2 py-[7px] text-[13px] font-medium transition-colors"
	class:bg-active={active}
	class:text-base-strong={active}
	class:text-base-dim={!active}
	class:bg-hover={!active}
>
	<Icon size={15} stroke={1.75} />
	<span class="flex-1 text-left">{label}</span>
	{#if count != null}
		<span class="text-base-mute font-mono text-[10.5px] tabular-nums">{count}</span>
	{/if}
</button>
