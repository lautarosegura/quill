<script lang="ts">
	import { onMount } from 'svelte';
	import { config } from '$lib/stores/config.svelte';
	import type { PromptPreset } from '$lib/types';

	onMount(() => {
		if (!config.value) config.load();
	});

	const PROMPT_MAX_CHARS = 880;

	const presets = $derived<PromptPreset[]>(config.value?.presets ?? []);
	const activeId = $derived<string | null>(config.value?.active_preset_id ?? null);

	// `null` selectedId means the "Sin preset" pseudo-row is selected.
	let selectedId = $state<string | null>(null);
	let initialized = false;

	$effect(() => {
		if (initialized) return;
		if (!config.value) return;
		initialized = true;
		// Land on the active preset if there is one, else first preset, else
		// "Sin preset".
		if (activeId) {
			selectedId = activeId;
		} else if (presets.length > 0) {
			selectedId = presets[0].id;
		} else {
			selectedId = null;
		}
	});

	const selected = $derived<PromptPreset | null>(
		selectedId ? presets.find((p) => p.id === selectedId) ?? null : null
	);

	// Built-ins come first (in their natural order), customs after a divider.
	const builtins = $derived(presets.filter((p) => p.builtin));
	const customs = $derived(presets.filter((p) => !p.builtin));

	async function setActive(id: string | null) {
		await config.set('active_preset_id', id);
	}

	function selectRow(id: string | null) {
		selectedId = id;
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
		if (activeId === selected.id) {
			await config.set('active_preset_id', null);
		}
		await config.set('presets', next);
		selectedId = next[0]?.id ?? null;
	}

	const promptCharCount = $derived(selected?.prompt.length ?? 0);
	const isSelectedActive = $derived(selected !== null && activeId === selected.id);
</script>

<div class="bg-app">
	<div class="mx-auto flex max-w-[920px] gap-6 p-8">
		<!-- ===== LEFT: master list ===== -->
		<aside
			class="border-hair bg-panel flex w-[280px] shrink-0 flex-col overflow-hidden rounded-lg border"
		>
			<div class="border-hair flex items-center justify-between border-b px-3 py-2.5">
				<div class="flex items-center gap-2">
					<span class="text-base-strong text-[12.5px] font-semibold">Presets</span>
					<span class="text-base-mute font-mono text-[10.5px]">{presets.length}</span>
				</div>
				<button class="ghost-btn" type="button" onclick={addCustomPreset}>
					<svg
						width="12"
						height="12"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
					>
						<path d="M12 5v14M5 12h14" />
					</svg>
					Nuevo
				</button>
			</div>

			<div class="flex flex-col gap-0.5 p-1.5">
				<!-- "Sin preset" pseudo-row -->
				<button
					class="preset-row"
					class:is-selected={selectedId === null}
					type="button"
					onclick={() => selectRow(null)}
					style:opacity={selectedId === null ? 1 : 0.85}
				>
					<span
						class="pradio"
						class:is-active={activeId === null}
						role="radio"
						aria-checked={activeId === null}
						tabindex="-1"
						onclick={(e) => {
							e.stopPropagation();
							setActive(null);
						}}
						onkeydown={(e) => {
							if (e.key === 'Enter' || e.key === ' ') {
								e.preventDefault();
								setActive(null);
							}
						}}
						aria-label="Sin preset activo"
					></span>
					<span class="text-base-dim flex-1 truncate text-[12.5px] italic">Sin preset</span>
				</button>

				{#if builtins.length > 0}
					<div class="my-1 mx-2 h-px" style="background: var(--border);"></div>
				{/if}

				{#each builtins as p (p.id)}
					<button
						class="preset-row"
						class:is-selected={selectedId === p.id}
						type="button"
						onclick={() => selectRow(p.id)}
					>
						<span
							class="pradio"
							class:is-active={activeId === p.id}
							role="radio"
							aria-checked={activeId === p.id}
							tabindex="-1"
							onclick={(e) => {
								e.stopPropagation();
								setActive(p.id);
							}}
							onkeydown={(e) => {
								if (e.key === 'Enter' || e.key === ' ') {
									e.preventDefault();
									setActive(p.id);
								}
							}}
							aria-label="Marcar como activo"
						></span>
						<div class="min-w-0 flex-1">
							<div class="flex items-center gap-1.5">
								<span class="text-base-strong truncate text-[12.5px] font-medium">{p.name}</span>
								<span class="badge-builtin">Built-in</span>
							</div>
						</div>
					</button>
				{/each}

				{#if customs.length > 0}
					<div class="my-1 mx-2 h-px" style="background: var(--border);"></div>
				{/if}

				{#each customs as p (p.id)}
					<button
						class="preset-row"
						class:is-selected={selectedId === p.id}
						type="button"
						onclick={() => selectRow(p.id)}
					>
						<span
							class="pradio"
							class:is-active={activeId === p.id}
							role="radio"
							aria-checked={activeId === p.id}
							tabindex="-1"
							onclick={(e) => {
								e.stopPropagation();
								setActive(p.id);
							}}
							onkeydown={(e) => {
								if (e.key === 'Enter' || e.key === ' ') {
									e.preventDefault();
									setActive(p.id);
								}
							}}
							aria-label="Marcar como activo"
						></span>
						<div class="min-w-0 flex-1">
							<div class="text-base-strong truncate text-[12.5px] font-medium">{p.name}</div>
						</div>
					</button>
				{/each}
			</div>
		</aside>

		<!-- ===== RIGHT: detail / editor ===== -->
		<div class="flex min-w-0 flex-1 flex-col gap-5">
			{#if !selected}
				<!-- "Sin preset" — empty detail -->
				<div
					class="border-hair bg-panel flex flex-col items-center gap-3 rounded-lg border px-6 py-12 text-center"
				>
					<div
						class="border-hair bg-elev text-base-mute flex h-9 w-9 items-center justify-center rounded-lg border"
					>
						<svg
							width="16"
							height="16"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="1.6"
							stroke-linecap="round"
							stroke-linejoin="round"
						>
							<rect x="3" y="6" width="18" height="14" rx="2" />
							<path d="M3 10h18M8 6V4M16 6V4" />
						</svg>
					</div>
					<div>
						<div class="text-base-strong text-[13px] font-medium">Ningún preset activo</div>
						<p
							class="text-base-dim mt-1 text-[11.5px]"
							style="max-width: 320px; text-wrap: pretty;"
						>
							Whisper recibe sólo tu vocabulario global. Activá un preset desde la lista o creá uno
							nuevo.
						</p>
					</div>
				</div>
			{:else}
				<!-- Header: name + active toggle -->
				<div class="flex items-start justify-between gap-4">
					<div>
						<div class="flex items-center gap-2">
							<h2 class="text-base-strong text-[18px] font-semibold tracking-tight">
								{selected.name}
							</h2>
							{#if selected.builtin}
								<span class="badge-builtin">Built-in</span>
							{/if}
						</div>
					</div>
					<button
						class="active-toggle"
						class:is-on={isSelectedActive}
						type="button"
						onclick={() => setActive(isSelectedActive ? null : selected.id)}
					>
						<span class="ind"></span>
						{isSelectedActive ? 'Activo' : 'Marcar activo'}
					</button>
				</div>

				<!-- Name field -->
				<div>
					<label class="label-eyebrow mb-1.5 block" for="preset-name">Nombre</label>
					<input
						id="preset-name"
						class="text-input"
						value={selected.name}
						disabled={selected.builtin}
						oninput={(e) => updateSelectedName(e.currentTarget.value)}
					/>
					{#if selected.builtin}
						<p class="text-base-mute mt-1 text-[10.5px]">
							Los nombres de built-ins no se pueden editar.
						</p>
					{/if}
				</div>

				<!-- Prompt field -->
				<div>
					<div class="mb-1.5 flex items-center justify-between">
						<label class="label-eyebrow" for="preset-prompt">Prompt</label>
						<span
							class="font-mono text-[10.5px]"
							style:color={promptCharCount > PROMPT_MAX_CHARS
								? 'oklch(0.72 0.18 25)'
								: 'var(--text-mute)'}
						>
							{promptCharCount} / {PROMPT_MAX_CHARS}
						</span>
					</div>
					<textarea
						id="preset-prompt"
						class="prompt-area"
						value={selected.prompt}
						oninput={(e) => updateSelectedPrompt(e.currentTarget.value)}
					></textarea>
					<p class="text-base-dim mt-2 text-[11px]" style="text-wrap: pretty;">
						El prompt se concatena con el vocabulario global y se pasa a Whisper como
						<span class="font-mono">--prompt</span> antes de cada transcripción.
					</p>
				</div>

				<!-- Footer: save status + delete -->
				<div class="border-hair flex items-center justify-between border-t pt-4">
					<span class="text-base-mute font-mono text-[10.5px]">Guardado</span>
					{#if selected.builtin}
						<span class="text-base-mute text-[11.5px]">Built-in · no se puede eliminar</span>
					{:else}
						<button
							class="ghost-btn"
							type="button"
							onclick={deleteSelected}
							style="color: oklch(0.72 0.18 25);"
						>
							<svg
								width="12"
								height="12"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="1.75"
								stroke-linecap="round"
								stroke-linejoin="round"
							>
								<path
									d="M3 6h18M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"
								/>
							</svg>
							Eliminar
						</button>
					{/if}
				</div>
			{/if}
		</div>
	</div>
</div>

<style>
	.preset-row {
		width: 100%;
		padding: 9px 10px 9px 12px;
		display: flex;
		align-items: center;
		gap: 10px;
		text-align: left;
		cursor: pointer;
		border-radius: 6px;
		transition: background 100ms ease;
		background: transparent;
		border: 0;
	}
	.preset-row:hover {
		background: var(--hover);
	}
	.preset-row.is-selected {
		background: var(--active);
		box-shadow: inset 2px 0 0 var(--accent);
	}

	.pradio {
		flex-shrink: 0;
		width: 16px;
		height: 16px;
		border-radius: 99px;
		border: 1.5px solid var(--border-strong);
		background: var(--bg-elev);
		display: inline-flex;
		align-items: center;
		justify-content: center;
		transition: all 120ms ease;
		cursor: pointer;
	}
	.pradio:hover {
		border-color: var(--text-mute);
	}
	.pradio.is-active {
		border-color: var(--accent);
		background: var(--accent);
		box-shadow: 0 0 0 3px var(--accent-ring);
	}
	.pradio.is-active::after {
		content: '';
		width: 5px;
		height: 5px;
		border-radius: 99px;
		background: white;
	}

	.active-toggle {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		height: 30px;
		padding: 0 11px 0 9px;
		border-radius: 7px;
		border: 1px solid var(--border-strong);
		background: var(--bg-elev);
		font-size: 12px;
		font-weight: 500;
		color: var(--text-dim);
		cursor: pointer;
		transition: all 140ms ease;
	}
	.active-toggle:hover {
		background: var(--hover);
		color: var(--text);
	}
	.active-toggle.is-on {
		border-color: color-mix(in oklch, oklch(0.72 0.16 155) 50%, transparent);
		background: oklch(0.72 0.16 155 / 0.12);
		color: oklch(0.78 0.16 155);
	}
	.active-toggle .ind {
		width: 14px;
		height: 14px;
		border-radius: 99px;
		border: 1.5px solid var(--text-mute);
		display: inline-flex;
		align-items: center;
		justify-content: center;
	}
	.active-toggle.is-on .ind {
		border-color: oklch(0.72 0.16 155);
		background: oklch(0.72 0.16 155);
	}
	.active-toggle.is-on .ind::after {
		content: '';
		display: block;
		width: 4px;
		height: 4px;
		border-radius: 99px;
		background: oklch(0.18 0.04 155);
	}

	.text-input {
		width: 100%;
		height: 32px;
		padding: 0 10px;
		background: var(--bg);
		border: 1px solid var(--border-strong);
		border-radius: 7px;
		color: var(--text);
		font-size: 12.5px;
		outline: none;
		transition:
			border-color 120ms ease,
			box-shadow 120ms ease;
	}
	.text-input:focus {
		border-color: color-mix(in oklch, var(--accent) 50%, var(--border-strong));
		box-shadow: 0 0 0 3px var(--accent-ring);
	}
	.text-input:disabled {
		color: var(--text-dim);
		background: var(--bg-panel);
		cursor: not-allowed;
	}

	.prompt-area {
		width: 100%;
		min-height: 220px;
		background: var(--bg);
		border: 1px solid var(--border-strong);
		border-radius: 8px;
		padding: 12px 14px;
		font-family: 'JetBrains Mono', monospace;
		font-size: 12.5px;
		line-height: 1.55;
		color: var(--text);
		resize: vertical;
		outline: none;
		transition:
			border-color 120ms ease,
			box-shadow 120ms ease;
	}
	.prompt-area:focus {
		border-color: color-mix(in oklch, var(--accent) 50%, var(--border-strong));
		box-shadow: 0 0 0 3px var(--accent-ring);
	}
	.prompt-area:disabled {
		color: var(--text-dim);
		background: var(--bg-panel);
	}
</style>
