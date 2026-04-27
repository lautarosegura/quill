<script lang="ts">
	import { onMount } from 'svelte';
	import { config } from '$lib/stores/config.svelte';
	import type { Substitution } from '$lib/types';

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

	// --- Substitutions ---

	const subs = $derived(config.value?.substitutions ?? []);

	function updateSubs(next: Substitution[]) {
		config.set('substitutions', next);
	}

	function addSub() {
		updateSubs([...subs, { from: '', to: '', case_sensitive: false }]);
	}

	function removeSub(i: number) {
		updateSubs(subs.filter((_, idx) => idx !== i));
	}

	function updateSubField<K extends keyof Substitution>(i: number, key: K, value: Substitution[K]) {
		updateSubs(subs.map((s, idx) => (idx === i ? { ...s, [key]: value } : s)));
	}
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

	<!-- ========== Substitutions ========== -->
	<h2 class="text-base-strong mt-10 text-[16px] font-semibold tracking-tight">Sustituciones</h2>
	<p class="text-base-dim mt-1 text-[13px]">
		Reemplazo exacto post-transcripción. Útil para errores que el vocabulario no logra evitar — Whisper
		se obstina en escribir <span class="font-mono">"Mokia"</span> y vos querés <span class="font-mono">"Nokia"</span>.
	</p>

	<div class="border-hair bg-panel mt-4 overflow-hidden rounded-lg border">
		{#if subs.length === 0}
			<div class="p-6 text-center">
				<p class="text-base-mute text-[12.5px]">
					Sin sustituciones aún. Agregá una para empezar.
				</p>
			</div>
		{:else}
			<table class="w-full text-[12.5px]">
				<thead class="border-hair bg-elev border-b">
					<tr>
						<th class="text-base-dim px-3 py-2 text-left text-[11px] font-medium uppercase tracking-wider">
							Reemplazar
						</th>
						<th class="text-base-dim px-3 py-2 text-left text-[11px] font-medium uppercase tracking-wider">
							Por
						</th>
						<th class="text-base-dim px-3 py-2 text-left text-[11px] font-medium uppercase tracking-wider">
							Aa
						</th>
						<th class="w-10"></th>
					</tr>
				</thead>
				<tbody>
					{#each subs as s, i (i)}
						<tr class="border-hair border-b last:border-b-0">
							<td class="px-2 py-1">
								<input
									type="text"
									class="bg-elev border-hair text-base-strong w-full rounded-md border px-2 py-1.5 font-mono focus:outline-none"
									placeholder="Mokia"
									value={s.from}
									oninput={(e) => updateSubField(i, 'from', e.currentTarget.value)}
								/>
							</td>
							<td class="px-2 py-1">
								<input
									type="text"
									class="bg-elev border-hair text-base-strong w-full rounded-md border px-2 py-1.5 font-mono focus:outline-none"
									placeholder="Nokia"
									value={s.to}
									oninput={(e) => updateSubField(i, 'to', e.currentTarget.value)}
								/>
							</td>
							<td class="px-3 py-1 text-center">
								<input
									type="checkbox"
									checked={s.case_sensitive}
									onchange={(e) => updateSubField(i, 'case_sensitive', e.currentTarget.checked)}
									title="Sensible a mayúsculas"
									aria-label="Sensible a mayúsculas"
								/>
							</td>
							<td class="px-1 py-1 text-center">
								<button
									type="button"
									class="text-base-mute hover:text-base-strong rounded px-2 py-1 text-[14px] transition-colors"
									onclick={() => removeSub(i)}
									title="Eliminar"
									aria-label="Eliminar"
								>
									×
								</button>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		{/if}

		<div class="border-hair flex items-center justify-end border-t p-2">
			<button
				type="button"
				class="text-base-dim hover:text-base-strong rounded px-3 py-1.5 text-[12px] transition-colors"
				onclick={addSub}
			>
				+ Agregar
			</button>
		</div>
	</div>

	<div class="border-hair bg-panel mt-4 rounded-lg border p-4">
		<div class="text-base-strong text-[12px] font-semibold">Cómo funciona</div>
		<p class="text-base-dim mt-2 text-[12px] leading-relaxed">
			Después de cada transcripción, Quill aplica estas reglas con <span class="font-mono">word boundaries</span>
			— "Mokia" se reemplaza pero "Mokian" no. Marcá <span class="font-mono">Aa</span> si necesitás que
			distinga mayúsculas / minúsculas (por defecto no las distingue).
		</p>
	</div>
</div>
