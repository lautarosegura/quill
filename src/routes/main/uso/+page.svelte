<script lang="ts">
	import { onMount } from 'svelte';
	import { open as openExternal } from '@tauri-apps/plugin-shell';
	import { BarChart } from '$lib/components/ui';
	import * as api from '$lib/ipc/commands';
	import type { UsageStats } from '$lib/ipc/commands';
	import { config } from '$lib/stores/config.svelte';

	let stats = $state<UsageStats | null>(null);
	let loading = $state(true);
	let alertInput = $state<string>('');

	onMount(async () => {
		loading = true;
		try {
			if (!config.value) await config.load();
			alertInput = config.value?.monthly_cost_alert_usd?.toString() ?? '';
			stats = await api.getUsageStats();
		} catch (e) {
			console.error('getUsageStats failed:', e);
		} finally {
			loading = false;
		}
	});

	function onAlertInput(v: string) {
		alertInput = v;
		const num = v.trim() === '' ? null : Number.parseFloat(v);
		const value = num != null && !Number.isNaN(num) && num > 0 ? num : null;
		config.setDebounced('monthly_cost_alert_usd', value);
	}

	function formatMinutes(seconds: number): string {
		const minutes = Math.floor(seconds / 60);
		const remainingSec = seconds % 60;
		if (minutes === 0) return `${seconds} s`;
		if (remainingSec === 0) return `${minutes} min`;
		return `${minutes} min ${remainingSec} s`;
	}

	function formatCost(usd: number): string {
		if (usd < 0.01) return '$0.00';
		if (usd < 1) return `$${usd.toFixed(3)}`;
		return `$${usd.toFixed(2)}`;
	}

	/** Human-friendly month label: "Abril 2026" */
	function formatMonth(ym: string): string {
		const [year, month] = ym.split('-').map((n) => parseInt(n, 10));
		const months = [
			'Enero', 'Febrero', 'Marzo', 'Abril', 'Mayo', 'Junio',
			'Julio', 'Agosto', 'Septiembre', 'Octubre', 'Noviembre', 'Diciembre'
		];
		return `${months[month - 1]} ${year}`;
	}

	const localPct = $derived.by(() => {
		if (!stats || stats.total_transcriptions === 0) return 0;
		return (stats.local_transcriptions / stats.total_transcriptions) * 100;
	});
	const groqPct = $derived(100 - localPct);

	const chartData = $derived.by(() => {
		if (!stats) return [];
		return stats.daily_counts.map((d) => {
			const date = new Date(d.date);
			return {
				label: date.toLocaleDateString('es-AR', { day: 'numeric', month: 'short' }),
				value: d.count,
				tooltip: `${date.toLocaleDateString('es-AR', { day: 'numeric', month: 'long' })}: ${d.count} dictado${d.count === 1 ? '' : 's'}`
			};
		});
	});

	const alertExceeded = $derived.by(() => {
		if (!stats || !config.value?.monthly_cost_alert_usd) return false;
		return stats.estimated_groq_cost_usd >= config.value.monthly_cost_alert_usd;
	});
</script>

<div class="mx-auto max-w-[860px] p-8">
	<div class="flex items-baseline justify-between">
		<div>
			<h1 class="text-base-strong text-[22px] font-semibold tracking-tight">Uso</h1>
			<p class="text-base-dim mt-1 text-sm">
				{stats ? formatMonth(stats.month) : 'Cargando estadísticas…'}
			</p>
		</div>
	</div>

	{#if loading}
		<div class="text-base-dim border-hair bg-panel mt-6 rounded-lg border p-8 text-center text-sm">
			Cargando…
		</div>
	{:else if !stats || stats.total_transcriptions === 0}
		<div class="text-base-dim border-hair bg-panel mt-6 rounded-lg border p-8 text-center text-sm">
			Sin dictados este mes todavía. Tus estadísticas aparecen acá al hacer el primero.
		</div>
	{:else}
		<!-- Stat cards -->
		<div class="mt-6 grid grid-cols-3 gap-3">
			<div class="border-hair bg-panel rounded-lg border p-4">
				<div class="text-base-mute text-[10.5px] font-semibold tracking-wider uppercase">
					Transcripciones
				</div>
				<div class="text-base-strong mt-1.5 text-[26px] font-semibold tracking-tight">
					{stats.total_transcriptions}
				</div>
				<div class="text-base-dim mt-1 font-mono text-[11px]">
					{stats.local_transcriptions} local · {stats.groq_transcriptions} groq
				</div>
			</div>

			<div class="border-hair bg-panel rounded-lg border p-4">
				<div class="text-base-mute text-[10.5px] font-semibold tracking-wider uppercase">
					Audio transcripto
				</div>
				<div class="text-base-strong mt-1.5 text-[26px] font-semibold tracking-tight">
					{formatMinutes(stats.total_audio_seconds)}
				</div>
				<div class="text-base-dim mt-1 font-mono text-[11px]">
					{formatMinutes(stats.groq_audio_seconds)} via Groq
				</div>
			</div>

			<div class="border-hair bg-panel rounded-lg border p-4">
				<div class="text-base-mute text-[10.5px] font-semibold tracking-wider uppercase">
					Costo estimado Groq
				</div>
				<div
					class="mt-1.5 text-[26px] font-semibold tracking-tight"
					class:accent-text={!alertExceeded}
					style={alertExceeded ? 'color: oklch(0.72 0.18 25);' : ''}
				>
					{formatCost(stats.estimated_groq_cost_usd)}
				</div>
				<div class="text-base-dim mt-1 font-mono text-[11px]">USD · este mes</div>
			</div>
		</div>

		<!-- Bar chart -->
		<div class="border-hair bg-panel mt-6 rounded-lg border p-5">
			<div class="flex items-baseline justify-between">
				<h2 class="text-base-strong text-[13px] font-semibold">Últimos 30 días</h2>
				<span class="text-base-mute font-mono text-[10.5px]">
					pico: {Math.max(...stats.daily_counts.map((d) => d.count))}
				</span>
			</div>
			<div class="mt-4">
				<BarChart data={chartData} height={90} />
			</div>
		</div>

		<!-- Engine split -->
		<div class="border-hair bg-panel mt-4 rounded-lg border p-5">
			<h2 class="text-base-strong text-[13px] font-semibold">Distribución por motor</h2>
			<div class="mt-3 space-y-2">
				<div>
					<div class="flex items-baseline justify-between text-[12px]">
						<span class="text-base-dim flex items-center gap-1.5">
							<span class="h-1.5 w-1.5 rounded-full" style="background: oklch(0.72 0.15 145);"
							></span>
							Local
						</span>
						<span class="text-base-strong font-mono">
							{stats.local_transcriptions} · {localPct.toFixed(0)}%
						</span>
					</div>
					<div
						class="mt-1 h-1.5 overflow-hidden rounded-full"
						style="background: var(--hover);"
					>
						<div
							class="h-full rounded-full"
							style="width: {localPct}%; background: oklch(0.72 0.15 145);"
						></div>
					</div>
				</div>
				<div>
					<div class="flex items-baseline justify-between text-[12px]">
						<span class="text-base-dim flex items-center gap-1.5">
							<span class="h-1.5 w-1.5 rounded-full" style="background: var(--accent);"
							></span>
							Groq
						</span>
						<span class="text-base-strong font-mono">
							{stats.groq_transcriptions} · {groqPct.toFixed(0)}%
						</span>
					</div>
					<div
						class="mt-1 h-1.5 overflow-hidden rounded-full"
						style="background: var(--hover);"
					>
						<div
							class="h-full rounded-full"
							style="width: {groqPct}%; background: var(--accent);"
						></div>
					</div>
				</div>
			</div>
		</div>

		<!-- Cost alert -->
		<div class="border-hair bg-panel mt-4 rounded-lg border p-5">
			<h2 class="text-base-strong text-[13px] font-semibold">Alerta de costo mensual</h2>
			<p class="text-base-dim mt-1 text-[12px]">
				Avisame si el gasto Groq del mes supera este monto. La notificación se dispara una vez
				por mes al cruzar el umbral.
			</p>
			<div class="mt-3 flex items-center gap-2">
				<span class="text-base-dim text-[13px]">USD</span>
				<input
					type="number"
					step="0.5"
					min="0"
					class="bg-elev border-hair text-base-strong w-[140px] rounded-md border px-3 py-2 text-[13px] focus:outline-none"
					placeholder="sin límite"
					value={alertInput}
					oninput={(e) => onAlertInput(e.currentTarget.value)}
				/>
				{#if alertExceeded}
					<span class="text-[12px]" style="color: oklch(0.72 0.18 25);">
						⚠️ Ya superaste el límite
					</span>
				{/if}
			</div>
		</div>

		<!-- Groq console link -->
		<div class="text-base-mute mt-6 text-center text-[11px]">
			Para consumo real y límites de tu cuenta:
			<button
				type="button"
				onclick={() => openExternal('https://console.groq.com/').catch(console.error)}
				class="hover:text-base-dim underline-offset-2 hover:underline"
			>
				console.groq.com ↗
			</button>
		</div>
	{/if}
</div>
