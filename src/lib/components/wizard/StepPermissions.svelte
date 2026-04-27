<script lang="ts">
	import { onMount } from 'svelte';
	import { open as openExternal } from '@tauri-apps/plugin-shell';
	import { Button, IconMic, IconCheck, IconAlert, IconCopy } from '$lib/components/ui';
	import {
		platform,
		initLinuxEnvironment,
		needsInputGroupSetup,
		type LinuxEnvironment
	} from '$lib/stores/platform.svelte';

	// Simple heuristic: this UI is only relevant on macOS. Most users are on
	// Windows. We skip the platform query and show both — Windows users just
	// see the green checks.
	const isMac = navigator.userAgent.toLowerCase().includes('mac');
	const isLinux = platform.isLinux;

	// Lazy: only Linux users hit the backend command. The result decides
	// whether we render the input-group setup card below.
	let linuxEnv = $state<LinuxEnvironment | null>(platform.linuxEnvironment);
	let copied = $state(false);
	const usermodCmd = 'sudo usermod -aG input $USER';

	onMount(() => {
		if (!isLinux) return;
		initLinuxEnvironment().then((env) => {
			linuxEnv = env;
		});
	});

	const showInputGroupCard = $derived(needsInputGroupSetup(linuxEnv));
	const compositorLabel = $derived(formatCompositor(linuxEnv));

	function formatCompositor(env: LinuxEnvironment | null): string {
		if (!env) return '';
		const desk = env.desktop;
		if (desk.toLowerCase() === 'gnome' && env.gnome_version) {
			return `GNOME ${env.gnome_version}`;
		}
		return desk;
	}

	async function copyUsermod() {
		try {
			await navigator.clipboard.writeText(usermodCmd);
			copied = true;
			setTimeout(() => (copied = false), 1600);
		} catch (err) {
			console.error('clipboard write failed', err);
		}
	}

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

		{#if showInputGroupCard}
			<!--
				Linux compositor without portal hotkey support (GNOME <48, Sway,
				Hyprland, wlroots…). Quill falls back to rdev's evdev listener,
				which needs read access to /dev/input/event* — typically gated by
				the `input` group on Debian / Ubuntu / Fedora.
			-->
			<div
				class="flex items-start gap-3 rounded-lg border p-4"
				style="border-color: oklch(0.72 0.15 70 / 0.4); background: oklch(0.72 0.15 70 / 0.06);"
			>
				<span class="mt-0.5 shrink-0" style="color: oklch(0.72 0.15 70);">
					<IconAlert size={16} />
				</span>
				<div class="min-w-0 flex-1">
					<div class="text-base-strong text-[13px] font-semibold">
						Atajo global en {compositorLabel || 'Wayland'}
					</div>
					<p class="text-base-dim mt-0.5 text-[12px] leading-relaxed">
						Tu compositor no expone el portal de atajos globales que Quill usa por defecto.
						Para que el atajo funcione, sumate al grupo <code
							class="rounded bg-black/20 px-1 py-0.5 text-[11px]">input</code
						>:
					</p>
					<div
						class="border-hair bg-panel mt-2 flex items-center gap-2 rounded-md border px-3 py-2"
					>
						<code class="text-base-strong flex-1 font-mono text-[12px]">{usermodCmd}</code>
						<button
							type="button"
							class="text-base-dim hover:text-base-strong shrink-0 transition-colors"
							onclick={copyUsermod}
							title="Copiar"
							aria-label="Copiar comando"
						>
							{#if copied}
								<IconCheck size={14} />
							{:else}
								<IconCopy size={14} />
							{/if}
						</button>
					</div>
					<p class="text-base-mute mt-2 text-[11px] leading-relaxed">
						Después tenés que cerrar sesión y volver a entrar para que el grupo se aplique. En
						compositores con portal (GNOME 48+, KDE Plasma 6) este paso no hace falta.
					</p>
				</div>
			</div>
		{/if}
	</div>
</div>
