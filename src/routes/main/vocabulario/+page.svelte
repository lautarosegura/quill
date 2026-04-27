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
		return 'var(--text-mute)';
	});

	// --- Substitutions ---

	const subs = $derived<Substitution[]>(config.value?.substitutions ?? []);

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

	function toggleCaseSensitive(i: number) {
		updateSubField(i, 'case_sensitive', !subs[i].case_sensitive);
	}
</script>

<div class="mx-auto max-w-[760px] p-8">
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
			Whisper le da más peso a esas palabras, lo que arregla cosas como
			<span class="font-mono">"Qil"</span> → <span class="font-mono">"Quill"</span>. El cambio se guarda
			automáticamente y aplica a la próxima dictación (sin reiniciar).
		</p>
	</div>

	<!-- ========== Substitutions ========== -->
	<h2 class="text-base-strong mt-12 text-[15px] font-semibold tracking-tight">Sustituciones</h2>
	<p
		class="text-base-dim mt-1 text-[12.5px] leading-relaxed"
		style="text-wrap: pretty; max-width: 560px;"
	>
		Reemplazo exacto post-transcripción. Útil para errores que el vocabulario no logra evitar —
		Whisper se obstina en escribir <span class="text-base-strong font-mono">"Mokia"</span> y vos
		querés
		<span class="text-base-strong font-mono">"Nokia"</span>.
	</p>

	{#if subs.length === 0}
		<!-- Empty state — accent CTA, centered card -->
		<div class="border-hair bg-panel mt-4 overflow-hidden rounded-lg border">
			<div class="flex flex-col items-center gap-3 px-6 py-10 text-center">
				<div
					class="border-hair bg-elev accent-text flex h-10 w-10 items-center justify-center rounded-lg border"
				>
					<svg
						width="18"
						height="18"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="1.6"
						stroke-linecap="round"
						stroke-linejoin="round"
					>
						<path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
						<path d="M8 10h8M8 14h5" />
					</svg>
				</div>
				<div>
					<div class="text-base-strong text-[13px] font-medium">Sin sustituciones aún</div>
					<p class="text-base-dim mt-1 text-[11.5px]" style="max-width: 280px; text-wrap: pretty;">
						Agregá una regla cuando notes un error recurrente que el vocabulario no logra evitar.
					</p>
				</div>
				<button class="accent-btn mt-1" type="button" onclick={addSub}>
					<svg
						width="13"
						height="13"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
					>
						<path d="M12 5v14M5 12h14" />
					</svg>
					Agregar primera regla
				</button>
			</div>
		</div>
	{:else}
		<!-- Table -->
		<div class="border-hair bg-panel mt-4 overflow-hidden rounded-lg border">
			<!-- Header -->
			<div
				class="border-hair bg-elev grid grid-cols-[1fr_1fr_44px_44px] items-center border-b px-3 py-2"
			>
				<div class="label-eyebrow">Reemplazar</div>
				<div class="label-eyebrow">Por</div>
				<div class="label-eyebrow tip text-center">
					Aa
					<span class="tip-body">Sensible a mayúsculas</span>
				</div>
				<div></div>
			</div>

			<!-- Rows -->
			{#each subs as s, i (i)}
				<div
					class="sub-row border-hair grid grid-cols-[1fr_1fr_44px_44px] items-center border-b last:border-b-0"
				>
					<div class="px-1">
						<input
							class="cell-input mono"
							value={s.from}
							placeholder="Mokia"
							oninput={(e) => updateSubField(i, 'from', e.currentTarget.value)}
						/>
					</div>
					<div class="px-1">
						<input
							class="cell-input mono"
							value={s.to}
							placeholder="(elimina la coincidencia)"
							oninput={(e) => updateSubField(i, 'to', e.currentTarget.value)}
						/>
					</div>
					<div class="flex items-center justify-center">
						<button
							class="aa-toggle"
							class:is-on={s.case_sensitive}
							type="button"
							onclick={() => toggleCaseSensitive(i)}
							title={s.case_sensitive ? 'Sensible a mayúsculas' : 'Insensible a mayúsculas'}
							aria-pressed={s.case_sensitive}
						>
							{#if s.case_sensitive}
								Aa
							{:else}
								<span>A</span><span class="lc">a</span>
							{/if}
						</button>
					</div>
					<div class="flex items-center justify-center">
						<button
							class="row-del"
							type="button"
							onclick={() => removeSub(i)}
							aria-label="Eliminar regla"
						>
							<svg
								width="13"
								height="13"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="1.75"
								stroke-linecap="round"
								stroke-linejoin="round"
							>
								<path
									d="M3 6h18M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6M10 11v6M14 11v6"
								/>
							</svg>
						</button>
					</div>
				</div>
			{/each}

			<!-- Footer -->
			<div class="border-hair bg-elev flex items-center justify-between border-t px-3 py-2">
				<span class="text-base-mute font-mono text-[10.5px]">
					{subs.length} {subs.length === 1 ? 'regla' : 'reglas'} · autosave
				</span>
				<button class="ghost-btn" type="button" onclick={addSub}>
					<svg
						width="13"
						height="13"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="1.75"
						stroke-linecap="round"
					>
						<path d="M12 5v14M5 12h14" />
					</svg>
					Agregar regla
				</button>
			</div>
		</div>

		<p class="text-base-mute mt-3 text-[11.5px]" style="text-wrap: pretty;">
			Pasá el cursor sobre una fila para ver el ícono de eliminar.
			<span class="font-mono">Aa</span> activa hace match exacto de mayúsculas.
		</p>
	{/if}
</div>

<style>
	.cell-input {
		width: 100%;
		height: 32px;
		padding: 0 10px;
		background: transparent;
		border: 0;
		outline: 0;
		font-size: 12.5px;
		color: var(--text);
		font-family: 'Inter', sans-serif;
	}
	.cell-input.mono {
		font-family: 'JetBrains Mono', monospace;
		font-size: 12px;
	}
	.cell-input::placeholder {
		color: var(--text-mute);
		font-style: italic;
	}
	.cell-input:focus {
		background: var(--hover);
	}

	.aa-toggle {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 24px;
		border-radius: 6px;
		border: 1px solid var(--border-strong);
		font-family: 'JetBrains Mono', monospace;
		font-size: 11px;
		font-weight: 600;
		color: var(--text-mute);
		background: transparent;
		cursor: pointer;
		user-select: none;
		transition: all 120ms ease;
	}
	.aa-toggle:hover {
		background: var(--hover);
		color: var(--text-dim);
	}
	.aa-toggle.is-on {
		background: var(--accent-soft);
		color: var(--accent);
		border-color: color-mix(in oklch, var(--accent) 35%, transparent);
	}
	.aa-toggle .lc {
		font-size: 9.5px;
		opacity: 0.6;
	}
	.aa-toggle.is-on .lc {
		opacity: 0.5;
	}

	.row-del {
		width: 24px;
		height: 24px;
		border-radius: 5px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		color: var(--text-mute);
		opacity: 0;
		transition:
			opacity 120ms ease,
			background 120ms ease;
		cursor: pointer;
		background: transparent;
		border: 0;
	}
	.sub-row:hover .row-del,
	.row-del:focus-visible {
		opacity: 1;
	}
	.row-del:hover {
		background: oklch(0.65 0.18 25 / 0.14);
		color: oklch(0.78 0.18 25);
	}

	.sub-row {
		transition: background 120ms ease;
	}
	.sub-row:hover {
		background: var(--hover);
	}
</style>
