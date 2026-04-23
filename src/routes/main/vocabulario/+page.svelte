<script lang="ts">
	import { onMount } from 'svelte';
	import { config } from '$lib/stores/config.svelte';

	onMount(() => {
		if (!config.value) config.load();
	});

	const MAX_CHARS = 900;
	const value = $derived(config.value?.vocabulary ?? '');
	const charCount = $derived(value.length);
	const charColor = $derived.by(() => {
		if (charCount > MAX_CHARS) return 'oklch(0.72 0.18 25)';
		if (charCount > MAX_CHARS - 100) return 'oklch(0.78 0.14 75)';
		return 'var(--text-dim)';
	});
</script>

<div class="mx-auto max-w-[720px] p-8">
	<h1 class="text-base-strong text-[22px] font-semibold tracking-tight">Vocabulario</h1>
	<p class="text-base-dim mt-1 text-sm">
		Palabras que Quill debería transcribir mejor — nombres propios, marcas, términos técnicos.
	</p>

	<div class="border-hair bg-panel mt-6 rounded-lg border p-4">
		<textarea
			class="bg-elev border-hair text-base-strong w-full rounded-md border p-3 font-mono text-[12.5px] leading-relaxed focus:outline-none"
			style="min-height: 260px; resize: vertical;"
			placeholder="Lautaro, Quill, Tauri, Svelte, Supabase, microservicios, Kubernetes..."
			value={value}
			oninput={(e) => config.setDebounced('vocabulary', e.currentTarget.value)}
		></textarea>

		<div class="mt-2 flex items-center justify-between">
			<p class="text-base-mute text-[11px]">
				💡 Separá con comas. Palabras que uses seguido aparecerán mejor transcritas.
			</p>
			<span class="font-mono text-[11px] tabular-nums" style="color: {charColor}">
				{charCount} / {MAX_CHARS}
			</span>
		</div>

		{#if charCount > MAX_CHARS}
			<p class="mt-2 text-[11.5px]" style="color: oklch(0.72 0.18 25)">
				Excediste el límite. Whisper sólo va a considerar los primeros {MAX_CHARS} caracteres.
			</p>
		{/if}
	</div>

	<div class="border-hair bg-panel mt-4 rounded-lg border p-4">
		<div class="text-base-strong text-[12px] font-semibold">Cómo funciona</div>
		<p class="text-base-dim mt-2 text-[12px] leading-relaxed">
			Este texto se pasa como <code class="font-mono">--prompt</code> al motor en cada transcripción.
			Whisper le da más peso a esas palabras, lo que arregla cosas como <span
				class="font-mono">"Qil"</span
			>
			→ <span class="font-mono">"Quill"</span>. El cambio se guarda automáticamente y aplica a la
			próxima dictación (sin reiniciar).
		</p>
	</div>
</div>
