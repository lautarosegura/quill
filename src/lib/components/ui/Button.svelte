<script lang="ts">
	import type { Snippet } from 'svelte';

	type Variant = 'primary' | 'secondary' | 'ghost' | 'danger';
	type Size = 'sm' | 'md';

	interface Props {
		variant?: Variant;
		size?: Size;
		disabled?: boolean;
		onclick?: () => void;
		children?: Snippet;
	}
	let { variant = 'secondary', size = 'md', disabled, onclick, children }: Props = $props();

	const SIZES: Record<Size, string> = {
		sm: 'h-7 px-3 text-[11.5px]',
		md: 'h-8 px-3 text-[12px]'
	};

	const VARIANT_CLASSES: Record<Variant, string> = {
		primary: 'text-white font-semibold',
		secondary: 'text-base-dim bg-hover font-medium border border-hair',
		ghost: 'text-base-dim bg-hover font-medium',
		danger: 'text-white font-semibold'
	};

	const bgStyle = $derived(
		variant === 'primary'
			? 'background: var(--accent)'
			: variant === 'danger'
				? 'background: oklch(0.6 0.18 25)'
				: ''
	);
</script>

<button
	{onclick}
	{disabled}
	class="inline-flex items-center justify-center rounded-md transition-colors disabled:cursor-not-allowed disabled:opacity-40 {SIZES[
		size
	]} {VARIANT_CLASSES[variant]}"
	style={bgStyle}
>
	{@render children?.()}
</button>
