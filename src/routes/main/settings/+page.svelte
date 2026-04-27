<script lang="ts">
	import { onMount } from 'svelte';
	import {
		Accordion,
		Button,
		Dropdown,
		KeyCapture,
		OverlayPositionPicker,
		PasswordInput,
		RadioCard,
		Segmented,
		Slider,
		Toggle,
		VuMeter
	} from '$lib/components/ui';
	import IconCpu from '$lib/components/ui/icons/IconCpu.svelte';
	import IconModels from '$lib/components/ui/icons/IconModels.svelte';
	import IconVocab from '$lib/components/ui/icons/IconVocab.svelte';
	import IconCommand from '$lib/components/ui/icons/IconCommand.svelte';
	import IconMic from '$lib/components/ui/icons/IconMic.svelte';
	import IconLayout from '$lib/components/ui/icons/IconLayout.svelte';
	import IconClock from '$lib/components/ui/icons/IconClock.svelte';
	import IconSettings from '$lib/components/ui/icons/IconSettings.svelte';
	import IconAlert from '$lib/components/ui/icons/IconAlert.svelte';
	import { open as openExternal } from '@tauri-apps/plugin-shell';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import * as api from '$lib/ipc/commands';
	import { config } from '$lib/stores/config.svelte';
	import { keybindText } from '$lib/utils/format';
	import type { Engine, Keybind, Language, OverlayPosition } from '$lib/types';

	const APP_VERSION = __APP_VERSION__;

	// Local editable state (debounced + saved through config store).
	let groqKeyInput = $state('');
	let groqKeyMasked = $state<string | null>(null);
	let editingKey = $state(false); // true = show input form; false = show "saved" state
	let testState = $state<'idle' | 'testing' | 'success' | 'error'>('idle');
	let testMessage = $state<string>('');
	let successTimer: ReturnType<typeof setTimeout> | null = null;

	function flashSuccess(msg: string) {
		testState = 'success';
		testMessage = msg;
		if (successTimer) clearTimeout(successTimer);
		successTimer = setTimeout(() => {
			if (testState === 'success') {
				testMessage = '';
				testState = 'idle';
			}
		}, 3000);
	}

	let micDevices = $state<api.MicDevice[]>([]);
	let localModels = $state<api.LocalModel[]>([]);

	// VU meter state for Settings → Micrófono.
	let micTesting = $state(false);
	let micLevel = $state(0);
	let micUnlisten: UnlistenFn | null = null;

	async function toggleMicTest() {
		if (micTesting) {
			await api.stopMicTest();
			micTesting = false;
			micLevel = 0;
			micUnlisten?.();
			micUnlisten = null;
			return;
		}
		// Subscribe first so we don't miss the first tick.
		micUnlisten = await listen<number>('mic_level', (event) => {
			micLevel = event.payload;
		});
		try {
			await api.startMicTest(currentMic || null);
			micTesting = true;
		} catch (e) {
			micUnlisten?.();
			micUnlisten = null;
			console.error('start_mic_test failed', e);
		}
	}

	$effect(() => {
		// Cleanup on unmount.
		return () => {
			if (micTesting) {
				api.stopMicTest().catch(() => {});
				micUnlisten?.();
			}
		};
	});

	onMount(async () => {
		if (!config.value) await config.load();
		groqKeyMasked = await api.getGroqKeyMasked();
		try {
			micDevices = await api.listMicDevices();
		} catch {
			micDevices = [];
		}
		try {
			localModels = await api.listLocalModels();
		} catch {
			localModels = [];
		}
	});

	// Wrappers to keep the templates clean.
	const currentEngine = $derived<Engine>(config.value?.engine ?? 'local');
	const currentLanguage = $derived<Language>(config.value?.language ?? 'es');
	const currentHotkey = $derived<Keybind>(
		config.value?.hotkey ?? { modifiers: ['ctrl', 'shift'], key: 'Space' }
	);
	const currentOverlayPos = $derived<OverlayPosition>(config.value?.overlay_position ?? 'bottom-center');
	const currentMaxDur = $derived(config.value?.max_duration_secs ?? 60);
	const currentLocalModel = $derived(config.value?.local_model_name ?? 'ggml-base');
	const currentMic = $derived(config.value?.mic_device ?? '');
	const currentBoot = $derived(config.value?.start_on_boot ?? false);
	const currentSounds = $derived(config.value?.sounds_enabled ?? false);
	const currentCostAlert = $derived(config.value?.monthly_cost_alert_usd ?? null);
	let costAlertDraft = $state<number>(10);
	$effect(() => {
		if (currentCostAlert != null) costAlertDraft = currentCostAlert;
	});

	async function testKey() {
		if (!groqKeyInput.trim()) return;
		testState = 'testing';
		testMessage = '';
		try {
			const result = await api.testGroqKey(groqKeyInput.trim());
			testState = result.valid ? 'success' : 'error';
			testMessage = result.message;
		} catch (e: unknown) {
			testState = 'error';
			testMessage = e instanceof Error ? e.message : String(e);
		}
	}

	async function saveKey() {
		if (!groqKeyInput.trim()) return;
		try {
			await api.setGroqKey(groqKeyInput.trim());
			groqKeyMasked = await api.getGroqKeyMasked();
			groqKeyInput = '';
			editingKey = false;
			flashSuccess('Clave guardada');
		} catch (e: unknown) {
			testState = 'error';
			testMessage = e instanceof Error ? e.message : String(e);
		}
	}

	async function deleteKey() {
		try {
			await api.deleteGroqKey();
			groqKeyMasked = null;
			groqKeyInput = '';
			editingKey = false;
			testState = 'idle';
			testMessage = '';
		} catch (e: unknown) {
			testState = 'error';
			testMessage = e instanceof Error ? e.message : String(e);
		}
	}

	function startEditingKey() {
		editingKey = true;
		groqKeyInput = '';
		testState = 'idle';
		testMessage = '';
	}

	function cancelEditingKey() {
		editingKey = false;
		groqKeyInput = '';
		testState = 'idle';
		testMessage = '';
	}

	const engineOptions = [
		{
			value: 'local' as const,
			title: 'Local (whisper.cpp)',
			description: 'Corre en tu máquina. Privado. Funciona offline.'
		},
		{
			value: 'groq' as const,
			title: 'Groq Cloud',
			description: 'Más rápido y preciso. Necesita internet y API key.'
		}
	];

	const languageOptions = [
		{ value: 'es' as const, label: '🇪🇸 Español' },
		{ value: 'en' as const, label: '🇺🇸 English' }
	];

	const modelDropdownOptions = $derived(
		localModels.length > 0
			? localModels.map((m) => ({ value: m.name, label: `${m.name} · ${formatMB(m.size_bytes)}` }))
			: [{ value: currentLocalModel, label: `${currentLocalModel} (no detectado en ~/.quill/models)` }]
	);

	const micDropdownOptions = $derived([
		{ value: '', label: 'Sistema por defecto' },
		...micDevices.map((m) => ({
			value: m.name,
			label: `${m.name}${m.is_default ? ' (default del sistema)' : ''}`
		}))
	]);

	function formatMB(bytes: number) {
		return `${(bytes / (1024 * 1024)).toFixed(0)} MB`;
	}

	// Dynamic summaries for each Accordion — shown inline so users see the
	// current state of each section without opening it.
	const motorSummary = $derived.by(() => {
		if (currentEngine === 'groq') {
			const m = config.value?.groq_model ?? 'whisper-large-v3-turbo';
			return `Groq · ${m}`;
		}
		return 'Local (whisper.cpp)';
	});

	const localModelSummary = $derived.by(() => {
		const installed = localModels.find((m) => m.name === currentLocalModel);
		if (!installed) return `${currentLocalModel} (no descargado)`;
		return `${currentLocalModel} · ${formatMB(installed.size_bytes)}`;
	});

	const languageSummary = $derived(currentLanguage === 'es' ? 'Español' : 'English');
	const hotkeySummary = $derived(keybindText(currentHotkey));
	const micSummary = $derived(currentMic || 'Sistema por defecto');

	const overlaySummary = $derived.by(() => {
		const map: Record<OverlayPosition, string> = {
			'bottom-center': 'Abajo centrado',
			'bottom-left': 'Abajo izquierda',
			'bottom-right': 'Abajo derecha',
			'top-center': 'Arriba centrado',
			'top-left': 'Arriba izquierda',
			'top-right': 'Arriba derecha'
		};
		return map[currentOverlayPos];
	});

	const durationSummary = $derived(`${currentMaxDur} segundos`);

	const generalSummary = $derived.by(() => {
		const boot = currentBoot ? 'Autostart on' : 'Autostart off';
		const sounds = currentSounds ? 'sonidos on' : 'sonidos off';
		return `${boot} · ${sounds}`;
	});

	const costAlertSummary = $derived.by(() => {
		if (currentCostAlert == null) return 'Desactivada';
		return `Avisar al cruzar $${currentCostAlert.toFixed(2)} USD`;
	});

	async function resetToDefaults() {
		if (!confirm('¿Restaurar todos los ajustes a los valores por defecto? Esto no borra tu historial ni tu clave de Groq.')) {
			return;
		}
		const def = await api.getDefaultHotkey();
		await config.save({
			language: 'es',
			engine: 'local',
			local_model_name: 'ggml-base',
			groq_model: 'whisper-large-v3-turbo',
			hotkey: def,
			language_cycle_hotkey: null,
			mic_device: null,
			overlay_position: 'bottom-center',
			max_duration_secs: 60,
			min_duration_ms: 250,
			start_on_boot: false,
			sounds_enabled: false,
			vocabulary: '',
			monthly_cost_alert_usd: null,
			wizard_version: config.value?.wizard_version ?? 1,
			substitutions: []
		});
	}
</script>

<div class="mx-auto max-w-[720px] p-8">
	<h1 class="text-base-strong text-[22px] font-semibold tracking-tight">Ajustes</h1>
	<p class="text-base-dim mt-1 text-sm">Configuración de Quill — los cambios se aplican al instante.</p>

	<div class="mt-8 flex flex-col gap-3">
		<!-- Motor de transcripción -->
		<Accordion title="Motor de transcripción" summary={motorSummary} icon={IconCpu} defaultOpen>
			<div class="flex flex-col gap-3 py-2">
				<RadioCard
					value={currentEngine}
					options={engineOptions}
					onChange={(v) => config.set('engine', v)}
				/>

				{#if currentEngine === 'groq'}
					<div class="border-hair bg-panel mt-2 flex flex-col gap-3 rounded-lg border p-4">
						<div class="flex items-start justify-between gap-3">
							<div class="min-w-0 flex-1">
								<div class="text-base-strong text-[13px] font-semibold">API Key de Groq</div>
								{#if groqKeyMasked && !editingKey}
									<div class="mt-1 flex items-center gap-2">
										<span
											class="accent-text font-mono text-[12px] font-medium"
											style="color: oklch(0.75 0.16 155)"
										>
											● Configurada
										</span>
										<span class="text-base-mute font-mono text-[11px]">{groqKeyMasked}</span>
									</div>
								{:else if !groqKeyMasked && !editingKey}
									<div class="text-base-mute mt-0.5 text-[11px]">No hay clave guardada</div>
								{:else}
									<div class="text-base-mute mt-0.5 text-[11px]">
										{groqKeyMasked ? 'Ingresá la nueva clave' : 'Pegá tu clave de Groq'}
									</div>
								{/if}
							</div>
							<button
								type="button"
								onclick={() => openExternal('https://groq.com/').catch(console.error)}
								class="text-base-dim hover:text-base-strong shrink-0 text-[11.5px] underline-offset-2 hover:underline"
							>
								Obtener una clave ↗
							</button>
						</div>

						{#if editingKey || !groqKeyMasked}
							<!-- State C: editing / initial setup -->
							<PasswordInput
								value={groqKeyInput}
								onChange={(v) => (groqKeyInput = v)}
								placeholder="gsk_..."
							/>
							<div class="flex items-center gap-2">
								<Button
									variant="secondary"
									size="sm"
									onclick={testKey}
									disabled={!groqKeyInput.trim() || testState === 'testing'}
								>
									{testState === 'testing' ? 'Probando…' : 'Probar'}
								</Button>
								<Button
									variant="primary"
									size="sm"
									onclick={saveKey}
									disabled={!groqKeyInput.trim()}
								>
									Guardar
								</Button>
								{#if groqKeyMasked && editingKey}
									<Button variant="ghost" size="sm" onclick={cancelEditingKey}>
										Cancelar
									</Button>
								{/if}
							</div>
						{:else}
							<!-- State B: key saved, not editing -->
							<div class="flex items-center gap-2">
								<Button variant="secondary" size="sm" onclick={startEditingKey}>
									Cambiar clave
								</Button>
								<Button variant="ghost" size="sm" onclick={deleteKey}>Borrar</Button>
							</div>
						{/if}

						{#if testMessage}
							<div
								class="text-[12px] font-medium"
								style="color: {testState === 'success'
									? 'oklch(0.75 0.16 155)'
									: testState === 'error'
										? 'oklch(0.72 0.18 25)'
										: 'var(--text-dim)'}"
							>
								{testMessage}
							</div>
						{/if}
					</div>
				{/if}
			</div>
		</Accordion>

		<!-- Modelo local -->
		<Accordion title="Modelo local" summary={localModelSummary} icon={IconModels}>
			<div class="flex flex-col gap-2 py-2">
				<Dropdown
					value={currentLocalModel}
					options={modelDropdownOptions}
					onChange={(v) => config.set('local_model_name', v)}
				/>
				<p class="text-base-mute text-[11px]">
					Modelos detectados en <code class="font-mono">~/.quill/models/</code>. Para bajar o
					borrar modelos, andá a <strong>Modelos</strong>.
				</p>
			</div>
		</Accordion>

		<!-- Idioma -->
		<Accordion title="Idioma primario" summary={languageSummary} icon={IconVocab}>
			<div class="py-2">
				<Segmented
					value={currentLanguage}
					options={languageOptions}
					onChange={(v) => config.set('language', v)}
				/>
			</div>
		</Accordion>

		<!-- Hotkey -->
		<Accordion title="Atajo de dictado" summary={hotkeySummary} icon={IconCommand}>
			<div class="flex flex-col gap-2 py-2">
				<KeyCapture
					value={currentHotkey}
					onChange={(v) => config.set('hotkey', v)}
					onResetDefault={async () => {
						const def = await api.getDefaultHotkey();
						config.set('hotkey', def);
					}}
				/>
				<p class="text-base-mute text-[11px]">
					Los cambios aplican al próximo atajo, sin reiniciar. Para chords con la tecla Windows
					(ej. Ctrl + Win), usá <strong>Restaurar default</strong> — Windows intercepta la tecla
					antes de que el captor pueda leerla.
				</p>
			</div>
		</Accordion>

		<!-- Micrófono -->
		<Accordion title="Micrófono" summary={micSummary} icon={IconMic}>
			<div class="flex flex-col gap-3 py-2">
				<div class="flex items-center gap-2">
					<div class="min-w-0 flex-1">
						<Dropdown
							value={currentMic}
							options={micDropdownOptions}
							onChange={(v) => config.set('mic_device', v === '' ? null : v)}
						/>
					</div>
					<Button
						variant={micTesting ? 'primary' : 'secondary'}
						size="sm"
						onclick={toggleMicTest}
					>
						{micTesting ? 'Detener' : 'Probar'}
					</Button>
				</div>
				<VuMeter level={micLevel} />
				<p class="text-base-mute text-[11px]">
					{#if micTesting}
						Hablá para ver el nivel — verde = óptimo, ámbar = fuerte, rojo = clipping.
					{:else}
						Cambiar el micrófono aplica en la próxima grabación, sin reiniciar.
					{/if}
				</p>
			</div>
		</Accordion>

		<!-- Posición del overlay -->
		<Accordion title="Posición del overlay" summary={overlaySummary} icon={IconLayout}>
			<div class="py-2">
				<OverlayPositionPicker
					value={currentOverlayPos}
					onChange={(v) => config.set('overlay_position', v)}
				/>
			</div>
		</Accordion>

		<!-- Duración -->
		<Accordion title="Duración máxima de grabación" summary={durationSummary} icon={IconClock}>
			<div class="py-2">
				<Slider
					value={currentMaxDur}
					min={10}
					max={120}
					step={5}
					unit="s"
					onChange={(v) => config.setDebounced('max_duration_secs', v)}
				/>
			</div>
		</Accordion>

		<!-- General -->
		<Accordion title="General" summary={generalSummary} icon={IconSettings}>
			<div class="flex flex-col gap-3 py-2">
				<Toggle
					value={currentBoot}
					label="Iniciar Quill al arrancar el sistema"
					onChange={(v) => config.set('start_on_boot', v)}
				/>
				<Toggle
					value={currentSounds}
					label="Sonidos al empezar y terminar dictado"
					onChange={(v) => config.set('sounds_enabled', v)}
				/>
			</div>
		</Accordion>

		<!-- Alertas de gasto -->
		<Accordion title="Alerta de gasto mensual" summary={costAlertSummary} icon={IconAlert}>
			<div class="flex flex-col gap-3 py-2">
				<Toggle
					value={currentCostAlert != null}
					label="Avisarme cuando cruce un umbral mensual"
					onChange={(on) =>
						config.set('monthly_cost_alert_usd', on ? costAlertDraft : null)}
				/>
				{#if currentCostAlert != null}
					<div class="flex items-center gap-2 ml-[46px]">
						<span class="text-base-dim text-[12px]">Umbral:</span>
						<span class="text-base-mute font-mono text-[12px]">$</span>
						<input
							type="number"
							min="0.10"
							step="0.10"
							value={costAlertDraft}
							oninput={(e) => {
								const n = parseFloat(e.currentTarget.value);
								if (!Number.isNaN(n) && n > 0) {
									costAlertDraft = n;
									config.setDebounced('monthly_cost_alert_usd', n);
								}
							}}
							class="bg-elev border-hair text-base-strong w-20 rounded border px-2 py-1 text-right font-mono text-[12px] focus:outline-none"
						/>
						<span class="text-base-mute font-mono text-[12px]">USD</span>
					</div>
					<p class="text-base-mute ml-[46px] text-[11px]">
						Se dispara una vez por mes cuando el costo estimado cruza el umbral.
					</p>
				{/if}
			</div>
		</Accordion>

		<!-- Footer: reset + version -->
		<div class="mt-4 flex items-center justify-between">
			<div class="text-base-mute text-[11.5px]">
				Los cambios se aplican al instante. Quill <span class="font-mono">v{APP_VERSION}</span>
			</div>
			<button
				type="button"
				onclick={resetToDefaults}
				class="h-8 rounded-md border px-3 text-[12px] font-medium transition-colors"
				style="border-color: oklch(0.65 0.18 25 / 0.35); color: oklch(0.72 0.18 25); background: oklch(0.65 0.18 25 / 0.06);"
			>
				Restaurar ajustes por defecto
			</button>
		</div>
	</div>
</div>
