<script lang="ts">
	import { onMount } from 'svelte';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { Button } from '$lib/components/ui';
	import IconFeather from '$lib/components/ui/icons/IconFeather.svelte';
	import StepWelcome from '$lib/components/wizard/StepWelcome.svelte';
	import StepPermissions from '$lib/components/wizard/StepPermissions.svelte';
	import StepEngine from '$lib/components/wizard/StepEngine.svelte';
	import StepSetup from '$lib/components/wizard/StepSetup.svelte';
	import StepTest from '$lib/components/wizard/StepTest.svelte';
	import * as api from '$lib/ipc/commands';
	import { wizard } from '$lib/stores/wizard.svelte';
	import { config } from '$lib/stores/config.svelte';
	import type { Engine } from '$lib/types';

	let finishing = $state(false);
	let finishError = $state<string | null>(null);

	// Tauri window handle for the custom titlebar's close button. Resolved
	// inside $effect so SSR doesn't choke on `window.__TAURI_INTERNALS__`.
	type AppWindow = ReturnType<typeof getCurrentWindow>;
	let appWindow: AppWindow | null = null;
	$effect(() => {
		appWindow = getCurrentWindow();
	});

	async function closeWizard() {
		await appWindow?.close();
	}
	async function minimizeWizard() {
		await appWindow?.minimize();
	}

	// Pull the platform-correct default hotkey from Rust on mount. The wizard
	// store seeds a Unix-style placeholder; on Windows the Rust default is
	// Ctrl + Win (modifier-only) and we want the wizard to reflect that.
	onMount(async () => {
		try {
			const cfg = await api.getConfig();
			wizard.patch('hotkey', cfg.hotkey);
			wizard.patch('language', cfg.language);
			wizard.patch('localModel', cfg.local_model_name);
		} catch {}
	});

	const TOTAL_STEPS = 5;

	const canContinue = $derived.by(() => {
		const d = wizard.draft;
		switch (d.step) {
			case 1:
				return true;
			case 2:
				return true;
			case 3:
				return d.engineChoice != null;
			case 4: {
				const needsLocal = d.engineChoice === 'local' || d.engineChoice === 'both';
				const needsGroq = d.engineChoice === 'groq' || d.engineChoice === 'both';
				const localOk = !needsLocal || d.localModelDownloaded;
				// Groq: allow continue if key is at least typed (tested ideal but not required —
				// wizard can finish with unverified key, user can fix from Settings).
				const groqOk = !needsGroq || d.groqKey.trim().length > 0;
				return localOk && groqOk;
			}
			case 5:
				return true;
			default:
				return false;
		}
	});

	const continueLabel = $derived.by(() => {
		if (wizard.draft.step === 5) return finishing ? 'Terminando…' : 'Terminar ✓';
		if (wizard.draft.step === 1) return 'Empezar →';
		return 'Continuar →';
	});

	async function onContinue() {
		// When advancing from Step 4 → Step 5, apply the user's choices to the
		// runtime so the live dictation test in Step 5 actually uses their
		// setup (engine, model, hotkey, Groq key). `wizard_version` stays at 0
		// so if they abandon now the wizard reappears on next launch.
		if (wizard.draft.step === 4) {
			try {
				await applyDraftToRuntime({ markComplete: false });
			} catch (e: unknown) {
				finishError = e instanceof Error ? e.message : String(e);
				return;
			}
		}
		if (wizard.draft.step < TOTAL_STEPS) {
			wizard.next();
			return;
		}
		await finish();
	}

	/** Pushes the wizard's draft choices into the backend runtime. Called
	 *  twice: when entering Step 5 (so the test dictation works) and at
	 *  finish (with markComplete=true). */
	async function applyDraftToRuntime({ markComplete }: { markComplete: boolean }) {
		const d = wizard.draft;
		if ((d.engineChoice === 'groq' || d.engineChoice === 'both') && d.groqKey.trim()) {
			await api.setGroqKey(d.groqKey.trim());
		}
		const engineField: Engine = d.engineChoice === 'groq' ? 'groq' : 'local';
		const existing = config.value ?? (await api.getConfig());
		const nextConfig = {
			...existing,
			engine: engineField,
			language: d.language,
			hotkey: d.hotkey,
			local_model_name: d.localModel,
			wizard_version: markComplete ? 1 : existing.wizard_version
		};
		await api.saveConfig(nextConfig);
	}

	async function finish() {
		finishing = true;
		finishError = null;
		try {
			await applyDraftToRuntime({ markComplete: true });
			// Hand off to Rust: close wizard, show main.
			await api.finishWizard();
		} catch (e: unknown) {
			finishError = e instanceof Error ? e.message : String(e);
			finishing = false;
		}
	}

	function onPrev() {
		if (wizard.draft.step > 1) wizard.prev();
	}
</script>

<div class="flex h-screen flex-col">
	<!-- Custom titlebar: serves as both window chrome (drag region + close/min)
	     and content header (brand + step counter). Matches the main window's
	     look so the user perceives Quill as a single visual system. -->
	<div
		class="border-hair bg-panel flex h-9 shrink-0 items-center border-b select-none"
		data-tauri-drag-region
	>
		<div class="flex h-full items-center gap-2 pr-3 pl-3">
			<div
				class="flex h-5 w-5 shrink-0 items-center justify-center rounded-md"
				style="background: var(--accent-soft); color: var(--accent)"
			>
				<IconFeather size={11} stroke={2} />
			</div>
			<span class="text-base-strong text-[12.5px] leading-none font-semibold tracking-tight">
				Quill
			</span>
			<span class="text-base-mute text-[11px] leading-none">Setup</span>
		</div>
		<div class="h-full flex-1" data-tauri-drag-region></div>
		<span class="text-base-mute pr-3 font-mono text-[11px] leading-none">
			Paso {wizard.draft.step} de {TOTAL_STEPS}
		</span>
		<div class="flex h-full">
			<button
				type="button"
				onclick={minimizeWizard}
				aria-label="Minimizar"
				class="titlebar-btn flex h-full w-[46px] items-center justify-center transition-colors"
			>
				<svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden="true">
					<rect x="0" y="4.5" width="10" height="1" fill="currentColor" />
				</svg>
			</button>
			<button
				type="button"
				onclick={closeWizard}
				aria-label="Cerrar"
				class="titlebar-btn titlebar-btn-close flex h-full w-[46px] items-center justify-center transition-colors"
			>
				<svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden="true">
					<path d="M0 0 L10 10 M10 0 L0 10" stroke="currentColor" stroke-width="1" />
				</svg>
			</button>
		</div>
	</div>

	<div class="flex min-h-0 flex-1 flex-col p-6">
		<!-- Progress dots: dot-line-dot-line-... Active dot expands to 20px so the
		     user can quickly read their position from the shape alone. -->
		<div class="flex items-center gap-1.5">
			{#each Array.from({ length: TOTAL_STEPS }, (_, i) => i + 1) as n, i}
				{@const active = n === wizard.draft.step}
				{@const done = n < wizard.draft.step}
				<span
					class="rounded-full transition-all duration-200"
					style:width={active ? '20px' : '6px'}
					style:height="6px"
					style:background={done || active ? 'var(--accent)' : 'var(--border-strong)'}
				></span>
				{#if i < TOTAL_STEPS - 1}
					<span
						class="h-px flex-1 transition-colors duration-200"
						style:background={done ? 'var(--accent)' : 'var(--border)'}
					></span>
				{/if}
			{/each}
		</div>

		<!-- Step content — scrolls when taller than the 500px window allows -->
		<div class="mt-6 min-h-0 flex-1 overflow-y-auto pr-1">
			{#if wizard.draft.step === 1}
				<StepWelcome />
			{:else if wizard.draft.step === 2}
				<StepPermissions />
			{:else if wizard.draft.step === 3}
				<StepEngine />
			{:else if wizard.draft.step === 4}
				<StepSetup />
			{:else if wizard.draft.step === 5}
				<StepTest />
			{/if}
		</div>

		<!-- Error banner -->
		{#if finishError}
			<div
				class="mt-3 rounded-md border p-3 text-[12px]"
				style="border-color: oklch(0.65 0.18 25 / 0.3); background: oklch(0.65 0.18 25 / 0.08); color: oklch(0.75 0.14 25);"
			>
				No pudimos terminar: {finishError}
			</div>
		{/if}

		<!-- Footer -->
		<div class="mt-4 flex items-center justify-between">
			<Button
				variant="ghost"
				onclick={onPrev}
				disabled={wizard.draft.step === 1 || finishing}
			>
				← Atrás
			</Button>
			<Button variant="primary" onclick={onContinue} disabled={!canContinue || finishing}>
				{continueLabel}
			</Button>
		</div>
	</div>
</div>

<style>
	:global(html, body) {
		background: var(--bg);
	}
	.titlebar-btn {
		color: var(--text-dim);
	}
	.titlebar-btn:hover {
		background: var(--hover);
		color: var(--text);
	}
	.titlebar-btn-close:hover {
		background: oklch(0.6 0.22 25);
		color: white;
	}
</style>
