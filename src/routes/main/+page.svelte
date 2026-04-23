<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';
	import {
		Button,
		EmptyState,
		HistoryEntry as HistoryEntryCard,
		IconMic,
		KeyCap,
		Modal,
		Spinner,
		Toast
	} from '$lib/components/ui';
	import IconSearch from '$lib/components/ui/icons/IconSearch.svelte';
	import * as api from '$lib/ipc/commands';
	import type { HistoryEntry } from '$lib/ipc/commands';
	import { EVENTS } from '$lib/ipc/events';
	import { config } from '$lib/stores/config.svelte';
	import { counts } from '$lib/stores/counts.svelte';
	import {
		dayBucket,
		DAY_BUCKET_LABEL,
		keybindLabels,
		keybindText,
		type DayBucket
	} from '$lib/utils/format';

	const PAGE_SIZE = 50;

	type EngineFilter = 'all' | 'local' | 'groq';

	let entries = $state<HistoryEntry[]>([]);
	let totalCount = $state(0);
	let searchQuery = $state('');
	let loading = $state(false);
	let showClearModal = $state(false);
	let toast = $state<{ kind: 'success' | 'info' | 'error'; message: string } | null>(null);
	let toastTimer: ReturnType<typeof setTimeout> | null = null;
	let searchTimer: ReturnType<typeof setTimeout> | null = null;
	let engineFilter = $state<EngineFilter>('all');
	let searchInputEl = $state<HTMLInputElement | null>(null);
	/** True until the initial load finishes OR returns zero rows. Drives the
	 *  "show spinner instead of empty state" decision on first paint. */
	let initialLoadPending = $state(true);

	async function loadInitial() {
		loading = true;
		try {
			const [list, count] = await Promise.all([
				api.listHistory(PAGE_SIZE, 0),
				api.countHistory()
			]);
			entries = list;
			totalCount = count;
			counts.refreshHistory();
		} catch (e) {
			flashToast('error', `Error cargando historial: ${String(e)}`);
		} finally {
			loading = false;
			initialLoadPending = false;
		}
	}

	/** Refreshes the visible window WITHOUT toggling the full-page loading
	 *  spinner — used by the `history_changed` listener so new rows slot in
	 *  smoothly. We re-fetch as many rows as we currently show so the user's
	 *  scroll position stays intact; any rows beyond that are already known
	 *  stale-but-correct and will refresh next time they scroll/filter. */
	async function refreshInPlace() {
		try {
			const targetSize = Math.max(PAGE_SIZE, entries.length);
			const [list, count] = await Promise.all([
				api.listHistory(targetSize, 0),
				api.countHistory()
			]);
			entries = list;
			totalCount = count;
		} catch {
			// Silent — the listener fires often; an error toast would be noisy.
		}
	}

	async function runSearch(q: string) {
		loading = true;
		try {
			entries = await api.searchHistory(q, PAGE_SIZE);
		} catch (e) {
			flashToast('error', `Error en búsqueda: ${String(e)}`);
		} finally {
			loading = false;
		}
	}

	function onSearchInput(v: string) {
		searchQuery = v;
		if (searchTimer) clearTimeout(searchTimer);
		// Clearing the search should feel instant — no debounce needed.
		if (searchQuery.trim() === '') {
			loadInitial();
			return;
		}
		searchTimer = setTimeout(() => {
			runSearch(searchQuery.trim());
		}, 300);
	}

	async function loadMore() {
		loading = true;
		try {
			const next = await api.listHistory(PAGE_SIZE, entries.length);
			entries = [...entries, ...next];
		} finally {
			loading = false;
		}
	}

	async function onCopy(text: string) {
		try {
			await navigator.clipboard.writeText(text);
			flashToast('success', 'Copiado al portapapeles');
		} catch (e) {
			flashToast('error', String(e));
		}
	}

	async function onReinject(id: number) {
		try {
			await api.reinjectHistoryEntry(id);
			flashToast('info', 'Re-insertado en la app activa');
		} catch (e) {
			flashToast('error', String(e));
		}
	}

	async function onRetry(id: number) {
		try {
			await api.retryHistoryEntry(id);
			flashToast('success', 'Re-transcripto e insertado');
			// Reload so the new success row appears and the failed row reflects
			// its now-missing WAV (Reintentar button disappears).
			await loadInitial();
		} catch (e) {
			flashToast('error', `Retry falló: ${String(e)}`);
		}
	}

	async function onDelete(id: number) {
		try {
			await api.deleteHistoryEntry(id);
			entries = entries.filter((e) => e.id !== id);
			totalCount = Math.max(0, totalCount - 1);
			counts.refreshHistory();
			flashToast('info', 'Dictado borrado');
		} catch (e) {
			flashToast('error', String(e));
		}
	}

	async function confirmClearAll() {
		showClearModal = false;
		try {
			await api.clearAllHistory();
			entries = [];
			totalCount = 0;
			counts.refreshHistory();
			flashToast('info', 'Historial borrado');
		} catch (e) {
			flashToast('error', String(e));
		}
	}

	function flashToast(kind: 'success' | 'info' | 'error', message: string) {
		toast = { kind, message };
		if (toastTimer) clearTimeout(toastTimer);
		toastTimer = setTimeout(() => (toast = null), 2500);
	}

	onMount(async () => {
		if (!config.value) await config.load();
		await loadInitial();
	});

	// Ctrl+F / Ctrl+K focus search (⌘F / ⌘K on macOS). In a $effect so cleanup
	// runs on unmount; onMount can't return cleanup when it's async.
	$effect(() => {
		const onKey = (e: KeyboardEvent) => {
			if ((e.ctrlKey || e.metaKey) && (e.key === 'f' || e.key === 'k')) {
				e.preventDefault();
				searchInputEl?.focus();
			}
		};
		window.addEventListener('keydown', onKey);
		return () => window.removeEventListener('keydown', onKey);
	});

	// Real-time refresh: whenever the backend persists a new row, pull the
	// head again so the user sees it without reloading the page.
	$effect(() => {
		let unlisten: (() => void) | null = null;
		listen(EVENTS.HISTORY_CHANGED, () => {
			// Only refresh when we're viewing the unfiltered head; during an
			// active search the user has a specific query and a live flip
			// would be jarring.
			if (searchQuery.trim() === '') {
				void refreshInPlace();
			}
		}).then((fn) => {
			unlisten = fn;
		});
		return () => unlisten?.();
	});

	async function newDictation() {
		try {
			await invoke('trigger_test_dictation');
		} catch (e) {
			flashToast('error', `No pudimos disparar el dictado: ${String(e)}`);
		}
	}

	// Client-side filter on the loaded window of entries. For very long
	// histories with a specific engine filter you may need to Load more until
	// the engine's results show up. Acceptable tradeoff for now — server-side
	// filtering would require a new command.
	const visibleEntries = $derived.by(() => {
		if (engineFilter === 'all') return entries;
		return entries.filter((e) => e.engine === engineFilter);
	});

	const FILTERS: Array<{ id: EngineFilter; label: string }> = [
		{ id: 'all', label: 'Todo' },
		{ id: 'local', label: 'On-device' },
		{ id: 'groq', label: 'Groq' }
	];

	function filterCount(id: EngineFilter): number {
		if (id === 'all') return entries.length;
		return entries.filter((e) => e.engine === id).length;
	}

	const canLoadMore = $derived(
		searchQuery.trim() === '' && entries.length < totalCount
	);

	const hotkeyText = $derived(
		config.value?.hotkey ? keybindText(config.value.hotkey) : 'tu atajo configurado'
	);
	const hotkeyLabels = $derived(
		config.value?.hotkey ? keybindLabels(config.value.hotkey) : []
	);

	type Group = { bucket: DayBucket; label: string; entries: HistoryEntry[]; wordCount: number };

	function countWords(text: string): number {
		const t = text.trim();
		return t ? t.split(/\s+/).length : 0;
	}

	const groupedEntries = $derived.by<Group[]>(() => {
		const order: DayBucket[] = ['hoy', 'ayer', 'semana', 'antes'];
		const map = new Map<DayBucket, HistoryEntry[]>();
		for (const e of visibleEntries) {
			const b = dayBucket(e.created_at);
			const list = map.get(b) ?? [];
			list.push(e);
			map.set(b, list);
		}
		return order
			.filter((b) => map.has(b))
			.map((b) => {
				const items = map.get(b)!;
				const wordCount = items.reduce((acc, e) => acc + countWords(e.text), 0);
				return { bucket: b, label: DAY_BUCKET_LABEL[b], entries: items, wordCount };
			});
	});

	const totalWordsLoaded = $derived(
		visibleEntries.reduce((acc, e) => acc + countWords(e.text), 0)
	);
</script>

<div class="mx-auto max-w-[860px] p-8">
	<div class="flex items-baseline justify-between gap-4">
		<div>
			<h1 class="text-base-strong text-[22px] font-semibold tracking-tight">Historial</h1>
			<p class="text-base-mute mt-1.5 text-[12.5px]">
				{#if totalCount > 0}
					{totalCount} transcripción{totalCount === 1 ? '' : 'es'}
					{#if totalWordsLoaded > 0}
						<span class="text-base-mute">·</span>
						{totalWordsLoaded.toLocaleString('es-AR')} palabras{visibleEntries.length < totalCount ? ' cargadas' : ''}
					{/if}
				{:else}
					Tus dictados guardados localmente.
				{/if}
			</p>
		</div>
		<div class="flex items-center gap-1.5">
			{#if totalCount > 0}
				<Button variant="ghost" size="sm" onclick={() => (showClearModal = true)}>
					Borrar todo
				</Button>
			{/if}
			<button
				type="button"
				onclick={newDictation}
				class="flex h-8 items-center gap-1.5 rounded-md px-3 text-[12px] font-medium text-white"
				style="background: var(--accent)"
			>
				<IconMic size={13} stroke={1.75} />
				Nueva
				<span class="ml-0.5 flex items-center gap-0.5">
					{#each hotkeyLabels as k (k)}
						<KeyCap>{k}</KeyCap>
					{/each}
				</span>
			</button>
		</div>
	</div>

	<!-- Search + filter -->
	{#if totalCount > 0}
		<div class="mt-5 flex items-center gap-3">
			<div class="bg-elev border-hair relative flex flex-1 items-center gap-2 rounded-md border px-3 py-2">
				<span class="text-base-mute"><IconSearch size={14} /></span>
				<input
					bind:this={searchInputEl}
					type="text"
					class="text-base-strong flex-1 bg-transparent text-[13px] placeholder:text-[color:var(--text-mute)] focus:outline-none"
					placeholder="Buscar en tus dictados..."
					value={searchQuery}
					oninput={(e) => onSearchInput(e.currentTarget.value)}
				/>
				{#if searchQuery}
					<button
						class="text-base-mute hover:text-base-dim text-[11px]"
						onclick={() => onSearchInput('')}
					>
						Limpiar
					</button>
				{:else}
					<span class="flex items-center gap-0.5">
						<KeyCap>⌃</KeyCap><KeyCap>F</KeyCap>
					</span>
				{/if}
			</div>

			<div class="bg-elev border-hair flex items-center gap-0.5 rounded-md border p-0.5">
				{#each FILTERS as f (f.id)}
					<button
						type="button"
						onclick={() => (engineFilter = f.id)}
						class="bg-hover flex h-7 items-center gap-1.5 rounded px-2.5 text-[12px] font-medium transition-colors"
						class:text-base-strong={engineFilter === f.id}
						class:text-base-mute={engineFilter !== f.id}
						style={engineFilter === f.id ? 'background: var(--active)' : ''}
					>
						{f.label}
						<span class="font-mono text-[10.5px] tabular-nums opacity-70">{filterCount(f.id)}</span>
					</button>
				{/each}
			</div>
		</div>
	{/if}

	<!-- List -->
	<div class="mt-6 flex flex-col gap-2">
		{#if initialLoadPending}
			<!-- First-load skeleton. Gate is `initialLoadPending` alone (not
			     `&& loading`) because the first render happens BEFORE onMount
			     flips `loading = true`; requiring both would let the empty
			     state flash through for one frame. -->
			<div class="flex flex-col items-center justify-center gap-3 py-16">
				<Spinner size={22} />
				<span class="text-base-mute text-[12px]">Cargando historial…</span>
			</div>
		{:else if totalCount === 0 && !loading && searchQuery === ''}
			<div class="border-hair bg-panel rounded-lg border p-4">
				{#snippet emptyDescription()}
					<span>Apretá</span>
					{#each hotkeyLabels as k, i (k + i)}
						{#if i > 0}
							<span class="text-base-mute text-[11px]">+</span>
						{/if}
						<KeyCap>{k}</KeyCap>
					{/each}
					<span>en cualquier app para grabar tu primer dictado.</span>
				{/snippet}
				{#snippet emptyDetails()}
					<span class="inline-flex items-center gap-1">
						<span class="h-1 w-1 rounded-full" style="background: var(--accent)"></span>
						Funciona en cualquier app
					</span>
					<span>·</span>
					<span>Privado por default</span>
				{/snippet}
				<EmptyState
					title="Sin dictados todavía"
					descriptionSnippet={emptyDescription}
					detailsSnippet={emptyDetails}
				>
					<IconMic size={32} stroke={1.5} />
				</EmptyState>
			</div>
		{:else if visibleEntries.length === 0 && !loading && searchQuery !== ''}
			<div class="text-base-dim border-hair bg-panel rounded-lg border p-8 text-center text-sm">
				Sin resultados para "{searchQuery}"
			</div>
		{:else if visibleEntries.length === 0 && engineFilter !== 'all'}
			<div class="text-base-dim border-hair bg-panel rounded-lg border p-8 text-center text-sm">
				Sin dictados de {engineFilter === 'local' ? 'On-device' : 'Groq'} en las últimas {entries.length} entradas cargadas.
			</div>
		{:else if searchQuery.trim() !== ''}
			<!-- Search results: flat list, no day grouping -->
			{#each visibleEntries as entry (entry.id)}
				<HistoryEntryCard {entry} {onCopy} {onReinject} {onRetry} {onDelete} />
			{/each}
		{:else}
			<!-- Day-grouped list with sticky headers. `top-0` is relative to the
			     <main> scroller in +layout.svelte. -->
			{#each groupedEntries as group (group.bucket)}
				<div
					class="border-hair sticky top-0 z-10 -mx-8 flex items-baseline gap-3 border-b px-8 py-2 backdrop-blur"
					style="background-color: color-mix(in oklch, var(--bg) 85%, transparent)"
				>
					<span class="text-base-dim text-[11px] font-semibold tracking-[0.08em] uppercase">
						{group.label}
					</span>
					<span class="text-base-mute font-mono text-[10.5px]">
						{group.entries.length} · {group.wordCount.toLocaleString('es-AR')} palabras
					</span>
				</div>
				{#each group.entries as entry (entry.id)}
					<HistoryEntryCard {entry} {onCopy} {onReinject} {onRetry} {onDelete} />
				{/each}
			{/each}
		{/if}

		{#if canLoadMore}
			<div class="mt-4 flex justify-center">
				<Button variant="secondary" onclick={loadMore} disabled={loading}>
					{loading ? 'Cargando…' : 'Cargar más'}
				</Button>
			</div>
		{/if}
	</div>
</div>

<!-- Confirm clear-all -->
{#if showClearModal}
	<Modal
		title="¿Borrar todo el historial?"
		description="Se van a eliminar los {totalCount} dictados guardados. Esta acción no se puede deshacer."
		confirmLabel="Borrar todo"
		cancelLabel="Cancelar"
		destructive
		onConfirm={confirmClearAll}
		onCancel={() => (showClearModal = false)}
	/>
{/if}

<!-- Transient toast -->
{#if toast}
	<div class="fixed right-6 bottom-6 z-40">
		<Toast kind={toast.kind} message={toast.message} />
	</div>
{/if}
