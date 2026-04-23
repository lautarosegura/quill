<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { IconCheck, KeyCap } from '$lib/components/ui';
	import { EVENTS, type TranscriptionState } from '$lib/ipc/events';
	import * as api from '$lib/ipc/commands';
	import type { HistoryEntry } from '$lib/ipc/commands';
	import { wizard } from '$lib/stores/wizard.svelte';
	import { formatDuration, keybindLabels } from '$lib/utils/format';

	let dictationState = $state<TranscriptionState>({ state: 'idle' });
	let prevState = $state<TranscriptionState['state']>('idle');
	let result = $state<HistoryEntry | null>(null);
	/** Captured at mount — any history row newer than this is from the wizard's
	 *  test session, not a leftover from before the user got to Step 5. */
	let mountedAt = $state<Date>(new Date());

	const hotkeyLabels = $derived(keybindLabels(wizard.draft.hotkey));

	const unlistens: UnlistenFn[] = [];

	onMount(async () => {
		mountedAt = new Date();
		unlistens.push(
			await listen<TranscriptionState>(EVENTS.TRANSCRIPTION_STATE_CHANGED, async (event) => {
				const next = event.payload;
				const justFinished =
					next.state === 'idle' &&
					(prevState === 'injecting' || prevState === 'transcribing');
				dictationState = next;
				prevState = next.state;
				if (justFinished) {
					try {
						const [latest] = await api.listHistory(1, 0);
						if (
							latest &&
							latest.status === 'success' &&
							new Date(latest.created_at) >= mountedAt
						) {
							result = latest;
						}
					} catch {
						// swallow — the user can still finish the wizard
					}
				}
			})
		);
	});

	onDestroy(() => {
		unlistens.forEach((fn) => fn());
	});

	const statusLabel = $derived.by(() => {
		switch (dictationState.state) {
			case 'idle':
				return result ? null : 'Apretá el atajo y decí algo';
			case 'recording':
				return 'Grabando — soltá cuando termines';
			case 'transcribing':
				return 'Transcribiendo…';
			case 'injecting':
				return 'Insertando en la app activa…';
			case 'cancelled':
				return 'Transcripción cancelada';
			case 'error':
				return `Error: ${dictationState.message}`;
			default:
				return null;
		}
	});

	const isActive = $derived(
		dictationState.state === 'recording' || dictationState.state === 'transcribing'
	);

	const resultWordCount = $derived(
		result?.text.trim() ? result.text.trim().split(/\s+/).length : 0
	);

	const resultMeta = $derived.by(() => {
		if (!result) return '';
		const parts: string[] = [];
		const dur = formatDuration(result.duration_ms);
		if (dur) parts.push(dur);
		parts.push(result.engine === 'local' ? 'On-device' : 'Groq');
		parts.push(result.language);
		if (resultWordCount > 0) parts.push(`${resultWordCount} palabras`);
		return parts.join(' · ');
	});
</script>

<div class="flex h-full flex-col items-center text-center">
	<h2 class="text-base-strong text-[17px] font-semibold tracking-tight">Probá tu primer dictado</h2>
	<p class="text-base-dim mt-1 text-[12px]">
		Mantené el atajo apretado y decí algo. Soltá para transcribir.
	</p>

	<div class="mt-5 flex items-center gap-1.5">
		{#each hotkeyLabels as label, i (label + i)}
			{#if i > 0}
				<span class="text-base-mute text-xs">+</span>
			{/if}
			<KeyCap size="big">{label}</KeyCap>
		{/each}
	</div>

	<!-- Result / status box: dashed border when empty, accent-soft when filled -->
	<div
		class="mt-4 flex w-full max-w-[460px] flex-1 items-center justify-center rounded-lg p-4 text-center"
		style:border={result ? '1.5px solid var(--accent)' : '1.5px dashed var(--border-strong)'}
		style:background={result ? 'var(--accent-soft)' : 'transparent'}
		class:ring-accent={isActive && !result}
	>
		{#if result}
			<div class="flex flex-col items-center gap-2">
				<span
					class="inline-flex h-5 items-center gap-1 rounded px-1.5 text-[9.5px] font-semibold tracking-[0.08em] uppercase"
					style="background: oklch(0.72 0.16 155 / 0.14); color: oklch(0.75 0.16 155);"
				>
					<IconCheck size={10} /> Transcrito
				</span>
				<p
					class="text-base-strong text-[14px] font-medium"
					style="text-wrap: balance"
				>
					"{result.text}"
				</p>
				<div class="text-base-mute font-mono text-[10.5px]">{resultMeta}</div>
			</div>
		{:else}
			<span
				class="text-[12.5px]"
				class:text-base-mute={dictationState.state === 'idle'}
				class:italic={dictationState.state === 'idle'}
				class:text-base-strong={dictationState.state !== 'idle'}
			>
				{statusLabel ?? 'Tu transcripción va a aparecer acá…'}
			</span>
		{/if}
	</div>

	<p class="text-base-mute mt-4 max-w-[420px] text-[11px] leading-relaxed">
		El texto aparece donde tengas el foco, no acá. Tocá "Terminar" cuando quieras seguir — si
		saltás este paso, todo queda configurado igual.
	</p>
</div>
