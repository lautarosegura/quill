<script lang="ts">
	import IconEye from './icons/IconEye.svelte';
	import IconEyeOff from './icons/IconEyeOff.svelte';

	interface Props {
		value: string;
		onChange: (next: string) => void;
		placeholder?: string;
		disabled?: boolean;
	}
	let { value, onChange, placeholder, disabled = false }: Props = $props();

	let shown = $state(false);
</script>

<div
	class="bg-elev border-hair quill-password flex items-center gap-2 rounded-md border px-3 py-2"
	class:opacity-40={disabled}
>
	{#if shown}
		<input
			type="text"
			{value}
			{placeholder}
			{disabled}
			oninput={(e) => onChange(e.currentTarget.value)}
			class="text-base-strong flex-1 border-none bg-transparent text-[12.5px] outline-none"
		/>
	{:else}
		<input
			type="password"
			{value}
			{placeholder}
			{disabled}
			oninput={(e) => onChange(e.currentTarget.value)}
			class="text-base-strong flex-1 border-none bg-transparent text-[12.5px] outline-none"
		/>
	{/if}
	<button
		type="button"
		{disabled}
		onclick={() => (shown = !shown)}
		aria-label={shown ? 'Hide password' : 'Show password'}
		class="text-base-dim hover:text-base-strong flex shrink-0 items-center justify-center rounded p-0.5 transition-colors disabled:cursor-not-allowed"
	>
		{#if shown}
			<IconEyeOff size={15} />
		{:else}
			<IconEye size={15} />
		{/if}
	</button>
</div>

<style>
	.quill-password:focus-within {
		box-shadow: 0 0 0 2px var(--accent-ring);
	}
</style>
