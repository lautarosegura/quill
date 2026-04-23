<script lang="ts">
	import { open as openExternal } from '@tauri-apps/plugin-shell';
	import { Button, IconMic, IconCheck } from '$lib/components/ui';

	// Simple heuristic: this UI is only relevant on macOS. Most users are on
	// Windows. We skip the platform query and show both — Windows users just
	// see the green checks.
	const isMac = navigator.userAgent.toLowerCase().includes('mac');

	function openMacPrivacy(panel: string) {
		openExternal(`x-apple.systempreferences:com.apple.preference.security?${panel}`).catch(
			console.error
		);
	}
</script>

<div class="flex h-full flex-col justify-center">
	<h2 class="text-base-strong text-[18px] font-semibold tracking-tight">Permisos del sistema</h2>
	<p class="text-base-dim mt-1 text-[13px]">
		Quill necesita acceso al micrófono. El resto varía según tu sistema operativo.
	</p>

	<div class="mt-6 flex flex-col gap-3">
		<!-- Microphone — always required -->
		<div
			class="border-hair bg-panel flex items-start gap-3 rounded-lg border p-4"
		>
			<span class="text-base-dim mt-0.5 shrink-0">
				<IconMic size={16} />
			</span>
			<div class="min-w-0 flex-1">
				<div class="text-base-strong text-[13px] font-semibold">Micrófono</div>
				<p class="text-base-dim mt-0.5 text-[12px] leading-relaxed">
					{#if isMac}
						macOS te va a pedir permiso la primera vez que dictes. Aceptalo.
					{:else}
						Windows te va a pedir permiso automáticamente. No hace falta configurar nada.
					{/if}
				</p>
			</div>
			<span class="shrink-0" style="color: oklch(0.72 0.15 145);">
				<IconCheck size={16} />
			</span>
		</div>

		{#if isMac}
			<div class="border-hair bg-panel flex items-start gap-3 rounded-lg border p-4">
				<div class="min-w-0 flex-1">
					<div class="text-base-strong text-[13px] font-semibold">Input Monitoring</div>
					<p class="text-base-dim mt-0.5 text-[12px] leading-relaxed">
						Necesario para que Quill detecte tu atajo global.
					</p>
				</div>
				<Button
					variant="secondary"
					size="sm"
					onclick={() => openMacPrivacy('Privacy_ListenEvent')}
				>
					Abrir Settings
				</Button>
			</div>

			<div class="border-hair bg-panel flex items-start gap-3 rounded-lg border p-4">
				<div class="min-w-0 flex-1">
					<div class="text-base-strong text-[13px] font-semibold">Accessibility</div>
					<p class="text-base-dim mt-0.5 text-[12px] leading-relaxed">
						Necesario para insertar texto automáticamente en la app activa.
					</p>
				</div>
				<Button
					variant="secondary"
					size="sm"
					onclick={() => openMacPrivacy('Privacy_Accessibility')}
				>
					Abrir Settings
				</Button>
			</div>

			<p class="text-base-mute mt-2 text-[11px] leading-relaxed">
				Tras otorgar estos permisos, puede que necesites reiniciar Quill. Phase 6 va a agregar
				validación automática.
			</p>
		{/if}
	</div>
</div>
