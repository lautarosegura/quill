<script lang="ts" module>
	export type NavId = 'historial' | 'settings' | 'vocabulario' | 'modelos' | 'uso';
</script>

<script lang="ts">
	import { onMount } from 'svelte';
	import SidebarNavItem from './SidebarNavItem.svelte';
	import IconHistory from './icons/IconHistory.svelte';
	import IconSettings from './icons/IconSettings.svelte';
	import IconVocab from './icons/IconVocab.svelte';
	import IconModels from './icons/IconModels.svelte';
	import IconUsage from './icons/IconUsage.svelte';
	import IconMic from './icons/IconMic.svelte';
	import IconPause from './icons/IconPause.svelte';
	import KeyCap from './KeyCap.svelte';
	import StatusDot from './StatusDot.svelte';
	import { status, subscribeToStatus } from '$lib/stores/status.svelte';
	import { counts } from '$lib/stores/counts.svelte';
	import { config } from '$lib/stores/config.svelte';
	import { keybindLabels } from '$lib/utils/format';

	interface Props {
		activeNav: NavId;
		onNavigate: (id: NavId) => void;
		onQuickRecord: () => void;
	}
	let { activeNav, onNavigate, onQuickRecord }: Props = $props();

	const NAV: Array<{ id: NavId; label: string; Icon: typeof IconHistory }> = [
		{ id: 'historial', label: 'Historial', Icon: IconHistory },
		{ id: 'settings', label: 'Ajustes', Icon: IconSettings },
		{ id: 'vocabulario', label: 'Vocabulario', Icon: IconVocab },
		{ id: 'modelos', label: 'Modelos', Icon: IconModels },
		{ id: 'uso', label: 'Uso', Icon: IconUsage }
	];

	onMount(() => {
		subscribeToStatus();
		counts.refreshHistory();
		counts.subscribe();
	});

	const vocabCount = $derived.by(() => {
		const v = config.value?.vocabulary ?? '';
		if (!v.trim()) return 0;
		return v.split(',').filter((t) => t.trim().length > 0).length;
	});

	function navCount(id: NavId): number | undefined {
		if (id === 'historial') return counts.history ?? undefined;
		if (id === 'vocabulario') return vocabCount > 0 ? vocabCount : undefined;
		return undefined;
	}

	// Live recording duration, in seconds. Starts when state enters Recording,
	// resets to null on exit. Updated once per second for the mm:ss readout.
	let recordingStartedAt = $state<number | null>(null);
	let elapsedSec = $state(0);

	$effect(() => {
		if (status.value.state === 'recording') {
			recordingStartedAt = performance.now();
			elapsedSec = 0;
			const id = setInterval(() => {
				if (recordingStartedAt != null) {
					elapsedSec = Math.floor((performance.now() - recordingStartedAt) / 1000);
				}
			}, 500);
			return () => clearInterval(id);
		}
		recordingStartedAt = null;
		elapsedSec = 0;
	});

	function fmtDuration(sec: number): string {
		const m = Math.floor(sec / 60);
		const s = sec % 60;
		return `${m}:${s.toString().padStart(2, '0')}`;
	}

	const STATUS_TEXT: Record<
		'idle' | 'recording' | 'transcribing' | 'injecting' | 'error' | 'cancelled',
		string
	> = {
		idle: 'Listo',
		recording: 'Grabando',
		transcribing: 'Transcribiendo',
		injecting: 'Insertando',
		error: 'Error',
		cancelled: 'Cancelada'
	};

	const footerText = $derived(STATUS_TEXT[status.value.state]);

	/** Keycaps rendered on the Quick Record button — reflects the configured
	 *  hotkey so the label matches reality after the user changes the combo. */
	const hotkeyCaps = $derived(
		config.value?.hotkey ? keybindLabels(config.value.hotkey) : []
	);

	const footerSub = $derived.by(() => {
		const s = status.value;
		if (s.state === 'recording') return fmtDuration(elapsedSec);
		if (s.state === 'transcribing') {
			const cfg = config.value;
			if (!cfg) return 'procesando';
			return cfg.engine === 'groq' ? cfg.groq_model : cfg.local_model_name;
		}
		if (s.state === 'injecting') return 'pegando en la app activa';
		if (s.state === 'error') return s.message ?? 'ver detalles';
		if (s.state === 'cancelled') return 'descartaste el audio';
		return 'Esperando atajo';
	});
</script>

<aside class="bg-panel border-hair flex w-[220px] shrink-0 flex-col border-r">
	<!-- Quick record — the sidebar's brand + theme toggle moved up to the
	     titlebar, so this button is now the first visible element. -->
	<div class="px-3 pt-3 pb-3">
		<button
			onclick={onQuickRecord}
			class="border-hair-strong bg-elev bg-hover text-base-dim flex w-full items-center gap-2 rounded-md border px-2.5 py-2 text-[12.5px] transition-colors"
		>
			<IconMic size={14} stroke={1.75} />
			<span class="flex-1 text-left">Probar dictado</span>
			<div class="flex items-center gap-0.5">
				{#each hotkeyCaps as k (k)}
					<KeyCap>{k}</KeyCap>
				{/each}
			</div>
		</button>
	</div>

	<!-- Nav -->
	<nav class="flex flex-1 flex-col gap-0.5 overflow-y-auto px-2">
		<div
			class="text-base-mute px-2 pt-2 pb-1 text-[10.5px] font-semibold tracking-[0.08em] uppercase"
		>
			Workspace
		</div>
		{#each NAV as item}
			<SidebarNavItem
				Icon={item.Icon}
				label={item.label}
				active={item.id === activeNav}
				count={navCount(item.id)}
				onClick={() => onNavigate(item.id)}
			/>
		{/each}
	</nav>

	<!-- Status footer -->
	<div class="border-hair border-t px-3 py-3">
		<div class="bg-elev border-hair flex items-center gap-2.5 rounded-md border px-2.5 py-2">
			<StatusDot state={status.value.state} />
			<div class="min-w-0 flex-1">
				<div
					class="text-[12px] font-medium leading-tight"
					class:accent-text={status.value.state === 'recording'}
					class:text-base-dim={status.value.state !== 'recording'}
				>
					{footerText}
				</div>
				<div class="text-base-mute mt-0.5 truncate font-mono text-[10.5px] leading-tight">
					{footerSub}
				</div>
			</div>
			{#if status.value.state === 'recording'}
				<button class="bg-hover text-base-dim rounded p-1" aria-label="Pausar">
					<IconPause size={12} />
				</button>
			{/if}
		</div>
	</div>
</aside>
