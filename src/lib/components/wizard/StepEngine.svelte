<script lang="ts">
	import { onMount } from 'svelte';
	import * as api from '$lib/ipc/commands';
	import type { HardwareProfile } from '$lib/ipc/commands';
	import { wizard, type EngineChoice } from '$lib/stores/wizard.svelte';

	let hardware = $state<HardwareProfile | null>(null);

	onMount(async () => {
		try {
			hardware = await api.detectHardware();
			// If user hasn't picked yet and hardware is detected, prefill the
			// recommended local model.
			if (hardware && !wizard.draft.localModel) {
				wizard.patch('localModel', hardware.recommended_model);
			}
		} catch {
			hardware = null;
		}
	});

	interface EngineOption {
		value: EngineChoice;
		label: string;
		tagline: string;
		description: string;
		specs: string[];
		badge?: string;
	}

	const options = $derived<EngineOption[]>([
		{
			value: 'local',
			label: 'Local',
			tagline: 'privado, offline',
			description: 'Corre en tu máquina. No se envía audio a la nube.',
			specs: [
				`Modelo: ${hardware?.recommended_model ?? 'ggml-base'}`,
				'~900 ms por 3 s de audio',
				'$0 / mes'
			],
			badge: 'Recomendado'
		},
		{
			value: 'groq',
			label: 'Groq Cloud',
			tagline: 'más rápido',
			description: 'API rápida y precisa. Necesita internet.',
			specs: [
				'Whisper Large v3 turbo',
				'~500 ms por 3 s',
				'~$0.10 / mes típico'
			]
		},
		{
			value: 'both',
			label: 'Ambos',
			tagline: 'flexible',
			description: 'Usá Groq cuando estés online, Local cuando no.',
			specs: ['Fallback automático', 'Elegís por sesión', 'Lo mejor de los dos']
		}
	]);

	function onSelect(v: EngineChoice) {
		wizard.patch('engineChoice', v);
	}
</script>

<div class="flex flex-col">
	<h2 class="text-base-strong text-[18px] font-semibold tracking-tight">Elegí tu motor</h2>
	<p class="text-base-dim mt-1 text-[13px]">
		Podés cambiar esto después en Ajustes.
	</p>

	{#if hardware}
		<div
			class="border-hair bg-elev mt-4 flex items-center gap-3 rounded-md border px-3 py-2"
		>
			<span class="text-[16px]">💻</span>
			<div class="text-base-dim min-w-0 flex-1 text-[11.5px]">
				Detectamos:
				<span class="text-base-strong font-medium">
					{hardware.os} {hardware.arch}{#if hardware.apple_silicon} · Apple Silicon{/if}
				</span>
				· {hardware.ram_gb} GB RAM · {hardware.cpu_cores} cores
			</div>
		</div>
	{/if}

	<div class="mt-3 flex flex-col gap-2">
		{#each options as opt (opt.value)}
			{@const selected = wizard.draft.engineChoice === opt.value}
			<button
				type="button"
				role="radio"
				aria-checked={selected}
				onclick={() => onSelect(opt.value)}
				class="w-full rounded-lg border p-3.5 text-left transition-colors"
				class:bg-elev={!selected}
				class:bg-hover={!selected}
				class:border-hair={!selected}
				style={selected
					? 'border-color: var(--accent); background: var(--accent-soft); box-shadow: 0 0 0 2px var(--accent-ring);'
					: ''}
			>
				<div class="flex items-start gap-3">
					<!-- Radio dot -->
					<span
						class="mt-0.5 flex h-4 w-4 shrink-0 items-center justify-center rounded-full border"
						style:border-color={selected ? 'var(--accent)' : 'var(--border-strong)'}
						style:background="var(--bg-elev)"
					>
						{#if selected}
							<span class="h-2 w-2 rounded-full" style="background: var(--accent)"></span>
						{/if}
					</span>
					<div class="flex-1">
						<div class="flex items-center gap-2">
							<span class="text-base-strong text-[13px] font-semibold">{opt.label}</span>
							{#if opt.badge}
								<span
									class="accent-soft accent-text rounded px-1.5 py-0.5 text-[9.5px] font-semibold tracking-[0.08em] uppercase"
								>
									{opt.badge}
								</span>
							{/if}
							<span class="text-base-mute text-[11px]">· {opt.tagline}</span>
						</div>
						<p class="text-base-dim mt-1 text-[11.5px] leading-snug">{opt.description}</p>
						<div
							class="text-base-mute mt-2 flex flex-wrap gap-x-3 gap-y-1 font-mono text-[10.5px]"
						>
							{#each opt.specs as spec}
								<span>{spec}</span>
							{/each}
						</div>
					</div>
				</div>
			</button>
		{/each}
	</div>
</div>
