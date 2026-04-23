<script lang="ts">
	import Button from './Button.svelte';
	import type { GroqModelEntry } from '$lib/ipc/commands';

	interface Props {
		model: GroqModelEntry;
		isActive: boolean;
		onSetActive: () => void;
	}
	let { model, isActive, onSetActive }: Props = $props();

	const priceLabel = $derived.by(() => {
		if (model.cost_per_hour_usd == null) return null;
		return `$${model.cost_per_hour_usd.toFixed(2)}/hr`;
	});

	const languagesLabel = $derived.by(() => {
		if (model.languages.length === 0) return null;
		if (model.languages.includes('multilingual')) return 'Multilingüe';
		if (model.languages.length === 1 && model.languages[0] === 'en') return 'Sólo inglés';
		return model.languages.join(' / ').toUpperCase();
	});

	const englishOnly = $derived(
		model.languages.length === 1 && model.languages[0] === 'en'
	);
</script>

<div
	class="border-hair bg-panel flex items-start gap-4 rounded-lg border p-4 transition-colors"
	class:ring-accent={isActive}
>
	<!-- Price/cost indicator in place of size circle -->
	<div class="flex h-10 w-14 shrink-0 items-center justify-center">
		{#if priceLabel}
			<span
				class="rounded-md px-2 py-1 font-mono text-[10.5px] font-semibold"
				style="background: var(--accent-soft); color: var(--accent);"
			>
				{priceLabel}
			</span>
		{:else}
			<span class="text-base-mute font-mono text-[10px]">—</span>
		{/if}
	</div>

	<!-- Info -->
	<div class="min-w-0 flex-1">
		<div class="flex flex-wrap items-center gap-2">
			<span class="text-base-strong text-[14px] font-semibold">{model.display_name}</span>
			<span class="text-base-mute font-mono text-[11px]">{model.name}</span>
			{#if isActive}
				<span
					class="rounded px-1.5 py-0.5 text-[10px] font-semibold tracking-wider uppercase"
					style="background: oklch(0.72 0.16 155 / 0.15); color: oklch(0.75 0.14 155);"
				>
					Activo
				</span>
			{/if}
			{#if model.kind === 'new_unknown'}
				<span
					class="rounded px-1.5 py-0.5 text-[10px] font-semibold tracking-wider uppercase"
					style="background: oklch(0.78 0.14 75 / 0.18); color: oklch(0.8 0.14 75);"
					title="Groq agregó este modelo después de nuestra última actualización"
				>
					Nuevo
				</span>
			{/if}
			{#if model.kind === 'catalog_only'}
				<span
					class="rounded px-1.5 py-0.5 text-[10px] font-semibold tracking-wider uppercase"
					style="background: var(--hover); color: var(--text-mute);"
					title="No pudimos verificar disponibilidad en vivo con Groq"
				>
					Sin verificar
				</span>
			{/if}
			{#if englishOnly}
				<span
					class="rounded px-1.5 py-0.5 text-[10px] font-semibold tracking-wider uppercase"
					style="background: oklch(0.65 0.18 25 / 0.12); color: oklch(0.72 0.14 25);"
				>
					EN
				</span>
			{/if}
		</div>
		<p class="text-base-dim mt-1 text-[12.5px] leading-relaxed">{model.description}</p>
		{#if languagesLabel && !englishOnly}
			<p class="text-base-mute mt-1 text-[11px]">{languagesLabel}</p>
		{/if}
	</div>

	<!-- Actions -->
	<div class="flex shrink-0 items-start">
		{#if isActive}
			<span class="text-base-mute text-[11.5px] font-medium">En uso</span>
		{:else}
			<Button variant="secondary" size="sm" onclick={onSetActive}>Usar</Button>
		{/if}
	</div>
</div>
