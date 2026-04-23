<script lang="ts">
	interface Props {
		value: boolean;
		onChange: (next: boolean) => void;
		label?: string;
		disabled?: boolean;
	}
	let { value, onChange, label, disabled = false }: Props = $props();

	function toggle() {
		if (disabled) return;
		onChange(!value);
	}

	const trackBg = $derived(value ? 'var(--accent)' : 'var(--border-strong)');
</script>

{#if label}
	<label class="flex cursor-pointer items-center gap-3" class:opacity-40={disabled}>
		<button
			type="button"
			role="switch"
			aria-checked={value}
			aria-label={label}
			{disabled}
			onclick={toggle}
			class="relative inline-flex shrink-0 items-center rounded-full transition-colors duration-150 disabled:cursor-not-allowed"
			style="width: 34px; height: 18px; background: {trackBg}"
		>
			<span
				class="absolute top-1/2 block rounded-full bg-white transition-transform duration-150"
				style="width: 14px; height: 14px; left: 2px; transform: translateY(-50%) translateX({value
					? '16px'
					: '0'}); box-shadow: 0 1px 2px oklch(0 0 0 / 0.25)"
			></span>
		</button>
		<span class="text-base-dim text-[13px]">{label}</span>
	</label>
{:else}
	<button
		type="button"
		role="switch"
		aria-checked={value}
		aria-label="Toggle"
		{disabled}
		onclick={toggle}
		class="relative inline-flex shrink-0 items-center rounded-full transition-colors duration-150 disabled:cursor-not-allowed disabled:opacity-40"
		style="width: 34px; height: 18px; background: {trackBg}"
	>
		<span
			class="absolute top-1/2 block rounded-full bg-white transition-transform duration-150"
			style="width: 14px; height: 14px; left: 2px; transform: translateY(-50%) translateX({value
				? '16px'
				: '0'}); box-shadow: 0 1px 2px oklch(0 0 0 / 0.25)"
		></span>
	</button>
{/if}
