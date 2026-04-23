<script lang="ts">
	import IconCheck from './icons/IconCheck.svelte';
	import IconAlert from './icons/IconAlert.svelte';
	import IconClipboard from './icons/IconClipboard.svelte';

	type Kind = 'success' | 'info' | 'error';

	interface Props {
		kind: Kind;
		message: string;
		action?: string;
		onAction?: () => void;
	}
	let { kind, message, action, onAction }: Props = $props();

	const CFG: Record<Kind, { color: string; bg: string; border: string }> = {
		success: {
			color: 'oklch(0.75 0.16 155)',
			bg: 'oklch(0.72 0.16 155 / 0.1)',
			border: 'oklch(0.72 0.16 155 / 0.25)'
		},
		info: {
			color: 'var(--text-dim)',
			bg: 'var(--bg-elev)',
			border: 'var(--border-strong)'
		},
		error: {
			color: 'oklch(0.72 0.18 25)',
			bg: 'oklch(0.65 0.18 25 / 0.1)',
			border: 'oklch(0.65 0.18 25 / 0.3)'
		}
	};
</script>

<div
	class="flex items-center gap-2.5 rounded-lg border px-3 py-2"
	style="background: {CFG[kind].bg}; border-color: {CFG[kind].border}; box-shadow: 0 8px 20px -6px oklch(0 0 0 / 0.2)"
>
	<span style="color: {CFG[kind].color}">
		{#if kind === 'success'}
			<IconCheck size={13} />
		{:else if kind === 'error'}
			<IconAlert size={13} />
		{:else}
			<IconClipboard size={13} />
		{/if}
	</span>
	<span
		class="text-[12px] font-medium"
		style="color: {kind === 'info' ? 'var(--text)' : CFG[kind].color}"
	>
		{message}
	</span>
	{#if action}
		<button
			class="ml-2 text-[11px] font-semibold"
			style="color: {CFG[kind].color}"
			onclick={onAction}
		>
			{action}
		</button>
	{/if}
</div>
