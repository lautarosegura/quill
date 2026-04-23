<script lang="ts" generics="T extends string">
	interface Props {
		value: T;
		options: Array<{ value: T; label: string }>;
		onChange: (next: T) => void;
	}
	let { value, options, onChange }: Props = $props();
</script>

<div
	class="bg-elev border-hair inline-flex items-center rounded-md border p-0.5"
	role="radiogroup"
>
	{#each options as opt (opt.value)}
		{@const selected = opt.value === value}
		<button
			type="button"
			role="radio"
			aria-checked={selected}
			onclick={() => onChange(opt.value)}
			class="rounded-[5px] px-2.5 py-1 text-[12px] font-medium transition-colors duration-[120ms]"
			class:bg-active={selected}
			class:text-base-strong={selected}
			class:text-base-dim={!selected}
			class:bg-hover={!selected}
		>
			{opt.label}
		</button>
	{/each}
</div>
