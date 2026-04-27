<script lang="ts">
	import { onMount } from 'svelte';
	import { config } from '$lib/stores/config.svelte';
	import type { PromptPreset } from '$lib/types';

	onMount(() => {
		if (!config.value) config.load();
	});

	// The four built-in presets are seeded in `Config::default()` on the
	// Rust side. If the user has never touched presets they'll always
	// see them here.
	const presets = $derived<PromptPreset[]>(config.value?.presets ?? []);
	const activeId = $derived<string | null>(config.value?.active_preset_id ?? null);

	let selectedId = $state<string | null>(null);

	// Auto-select the active preset on first load so the user lands on
	// something meaningful. Falls back to the first preset if no active.
	$effect(() => {
		if (selectedId !== null) return;
		if (activeId) {
			selectedId = activeId;
		} else if (presets.length > 0) {
			selectedId = presets[0].id;
		}
	});

	const selected = $derived<PromptPreset | null>(
		selectedId ? presets.find((p) => p.id === selectedId) ?? null : null
	);

	async function setActive(id: string | null) {
		await config.set('active_preset_id', id);
	}

	async function updateSelectedPrompt(newPrompt: string) {
		if (!selected) return;
		const next = presets.map((p) => (p.id === selected.id ? { ...p, prompt: newPrompt } : p));
		await config.set('presets', next);
	}

	async function updateSelectedName(newName: string) {
		if (!selected) return;
		const next = presets.map((p) => (p.id === selected.id ? { ...p, name: newName } : p));
		await config.set('presets', next);
	}

	async function addCustomPreset() {
		// Generate a unique id from a slugged name, falling back to a counter.
		let suffix = 1;
		let id = `custom-${suffix}`;
		while (presets.some((p) => p.id === id)) {
			suffix += 1;
			id = `custom-${suffix}`;
		}
		const newPreset: PromptPreset = {
			id,
			name: `Custom ${suffix}`,
			prompt: '',
			builtin: false
		};
		await config.set('presets', [...presets, newPreset]);
		selectedId = id;
	}

	async function deleteSelected() {
		if (!selected || selected.builtin) return;
		const next = presets.filter((p) => p.id !== selected.id);
		// If the deleted preset was active, fall back to "no preset".
		if (activeId === selected.id) {
			await config.set('active_preset_id', null);
		}
		await config.set('presets', next);
		selectedId = next[0]?.id ?? null;
	}
</script>

<div class="mx-auto flex max-w-[920px] gap-6 p-8">
	<!-- Sidebar list -->
	<div class="border-hair bg-panel w-[260px] shrink-0 overflow-hidden rounded-lg border">
		<div class="border-hair flex items-center justify-between border-b px-3 py-2.5">
			<span class="text-base-strong text-[12.5px] font-semibold">Presets</span>
			<button
				type="button"
				class="text-base-dim hover:text-base-strong rounded px-2 py-1 text-[11px] transition-colors"
				onclick={addCustomPreset}
			>
				+ Nuevo
			</button>
		</div>

		<!-- "Sin preset" pseudo-row at top -->
		<button
			type="button"
			class="hover:bg-elev flex w-full items-center justify-between px-3 py-2 text-left transition-colors"
			class:active-row={activeId === null && selectedId === null}
			onclick={() => {
				selectedId = null;
			}}
		>
			<span class="text-base-dim text-[12.5px]">Sin preset</span>
			<input
				type="radio"
				name="active-preset"
				checked={activeId === null}
				onchange={() => setActive(null)}
				onclick={(e) => e.stopPropagation()}
			/>
		</button>

		{#each presets as p (p.id)}
			<button
				type="button"
				class="hover:bg-elev border-hair flex w-full items-center justify-between border-t px-3 py-2 text-left transition-colors"
				class:active-row={selectedId === p.id}
				onclick={() => {
					selectedId = p.id;
				}}
			>
				<div class="min-w-0 flex-1">
					<div class="text-base-strong truncate text-[12.5px]">{p.name}</div>
					{#if p.builtin}
						<div class="text-base-mute mt-0.5 text-[10.5px]">Built-in</div>
					{/if}
				</div>
				<input
					type="radio"
					name="active-preset"
					checked={activeId === p.id}
					onchange={() => setActive(p.id)}
					onclick={(e) => e.stopPropagation()}
					title="Marcar como activo"
				/>
			</button>
		{/each}
	</div>

	<!-- Detail / editor -->
	<div class="min-w-0 flex-1">
		<h1 class="text-base-strong text-[22px] font-semibold tracking-tight">Presets de prompt</h1>
		<p class="text-base-dim mt-1 text-sm">
			Cada preset le da a Whisper un contexto distinto — formal para email, casual para WhatsApp,
			técnico para código. El preset activo se concatena con tu Vocabulario al transcribir.
		</p>

		{#if !selected}
			<div class="border-hair bg-panel mt-6 rounded-lg border p-6 text-center">
				<p class="text-base-mute text-[12.5px]">
					Sin preset seleccionado. El prompt de Whisper va a ser solo tu Vocabulario.
				</p>
			</div>
		{:else}
			<div class="border-hair bg-panel mt-6 space-y-4 rounded-lg border p-4">
				<!-- Name -->
				<label class="block">
					<span class="text-base-dim mb-1 block text-[11px] font-medium uppercase tracking-wider">
						Nombre
					</span>
					<input
						type="text"
						class="bg-elev border-hair text-base-strong w-full rounded-md border px-3 py-2 text-[13px] focus:outline-none disabled:opacity-50"
						value={selected.name}
						disabled={selected.builtin}
						oninput={(e) => updateSelectedName(e.currentTarget.value)}
					/>
					{#if selected.builtin}
						<p class="text-base-mute mt-1 text-[10.5px]">
							El nombre de los built-in no se puede cambiar.
						</p>
					{/if}
				</label>

				<!-- Prompt -->
				<label class="block">
					<span class="text-base-dim mb-1 block text-[11px] font-medium uppercase tracking-wider">
						Prompt
					</span>
					<textarea
						class="bg-elev border-hair text-base-strong w-full rounded-md border p-3 font-mono text-[12.5px] leading-relaxed focus:outline-none"
						style="min-height: 220px; resize: vertical;"
						value={selected.prompt}
						oninput={(e) => updateSelectedPrompt(e.currentTarget.value)}
					></textarea>
					<p class="text-base-mute mt-1 text-[10.5px]">
						Whisper acepta ~224 tokens (~880 chars). Si combinás preset + vocabulario y se pasa,
						truncamos vocabulario primero (preservamos el preset).
					</p>
				</label>

				<!-- Active toggle + delete -->
				<div class="border-hair flex items-center justify-between border-t pt-3">
					<button
						type="button"
						class="text-[12px] transition-colors"
						class:text-accent={activeId === selected.id}
						class:text-base-dim={activeId !== selected.id}
						onclick={() => setActive(activeId === selected.id ? null : selected.id)}
					>
						{activeId === selected.id ? '✓ Preset activo' : 'Marcar como activo'}
					</button>
					{#if !selected.builtin}
						<button
							type="button"
							class="text-base-mute hover:text-[oklch(0.72_0.18_25)] text-[12px] transition-colors"
							onclick={deleteSelected}
						>
							Eliminar
						</button>
					{/if}
				</div>
			</div>

			<div class="border-hair bg-panel mt-4 rounded-lg border p-4">
				<div class="text-base-strong text-[12px] font-semibold">Cómo funciona</div>
				<p class="text-base-dim mt-2 text-[12px] leading-relaxed">
					Marcá un preset como activo, dictá normalmente. El prompt del preset le da a Whisper el
					"tono" o estilo, y tu Vocabulario lista las palabras específicas. Si no querés ningún
					preset, elegí <span class="font-mono">Sin preset</span> arriba — solo se usa el Vocabulario.
					También podés cambiar el preset activo desde el menú del tray sin abrir esta página.
				</p>
			</div>
		{/if}
	</div>
</div>

<style>
	.active-row {
		background: var(--elev);
	}
	.text-accent {
		color: var(--accent);
	}
</style>
