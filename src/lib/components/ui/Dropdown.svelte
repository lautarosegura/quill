<script lang="ts" generics="T extends string">
	import IconChevron from './icons/IconChevron.svelte';

	interface Props {
		value: T;
		options: Array<{ value: T; label: string }>;
		onChange: (next: T) => void;
		placeholder?: string;
		disabled?: boolean;
	}
	let { value, options, onChange, placeholder, disabled = false }: Props = $props();

	const showPlaceholder = $derived(placeholder !== undefined && value === '');
</script>

<div class="relative inline-block w-full">
	<select
		{disabled}
		value={value as string}
		onchange={(e) => onChange(e.currentTarget.value as T)}
		class="bg-elev border-hair text-base-strong quill-dropdown w-full cursor-pointer rounded-md border px-3 py-2 pr-9 text-[12.5px] transition-shadow focus:outline-none disabled:cursor-not-allowed disabled:opacity-40"
		class:is-placeholder={showPlaceholder}
	>
		{#if placeholder !== undefined}
			<option value="" disabled hidden={value !== ('' as T)}>{placeholder}</option>
		{/if}
		{#each options as opt (opt.value)}
			<option value={opt.value}>{opt.label}</option>
		{/each}
	</select>
	<span
		class="text-base-dim pointer-events-none absolute top-1/2 right-2.5 flex -translate-y-1/2 items-center"
		style="transform: translateY(-50%) rotate(90deg)"
		aria-hidden="true"
	>
		<IconChevron size={14} />
	</span>
</div>

<style>
	.quill-dropdown {
		appearance: none;
		-webkit-appearance: none;
		-moz-appearance: none;
	}
	.quill-dropdown::-ms-expand {
		display: none;
	}
	.quill-dropdown:focus {
		box-shadow: 0 0 0 2px var(--accent-ring);
	}
	.quill-dropdown.is-placeholder {
		color: var(--text-mute);
	}
	.quill-dropdown option {
		background: var(--bg-elev);
		color: var(--text);
	}
</style>
