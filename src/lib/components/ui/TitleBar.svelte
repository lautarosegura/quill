<script lang="ts">
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { theme } from '$lib/stores/theme.svelte';
	import IconFeather from './icons/IconFeather.svelte';
	import IconSun from './icons/IconSun.svelte';
	import IconMoon from './icons/IconMoon.svelte';

	// Initialize to `null` and resolve inside `$effect` so `getCurrentWindow()`
	// only runs in the browser. Calling it at module scope crashes SvelteKit's
	// SSR pass (it reads `window.__TAURI_INTERNALS__` synchronously).
	type AppWindow = ReturnType<typeof getCurrentWindow>;
	let appWindow: AppWindow | null = null;
	let maximized = $state(false);

	$effect(() => {
		const w = getCurrentWindow();
		appWindow = w;
		let unlisten: (() => void) | null = null;
		w.isMaximized().then((v) => {
			maximized = v;
		});
		w.onResized(async () => {
			maximized = await w.isMaximized();
		}).then((fn) => {
			unlisten = fn;
		});
		return () => unlisten?.();
	});

	async function minimize() {
		await appWindow?.minimize();
	}
	async function toggleMax() {
		await appWindow?.toggleMaximize();
	}
	async function close() {
		// Hits the CloseRequested handler in Rust, which hides-to-tray
		// instead of actually quitting.
		await appWindow?.close();
	}
</script>

<!--
  Full-width top strip that doubles as the window chrome. The drag region is
  applied to the root + the middle spacer; the brand block and window controls
  are interactive so they sit outside the drag surface.
-->
<div
	class="border-hair bg-panel relative flex h-9 shrink-0 items-center border-b select-none"
	data-tauri-drag-region
>
	<!-- Brand: Quill logo + name + version + theme toggle. Mirrors Discord's
	     "workspace identity at top-left" pattern. `leading-none` on the text
	     spans removes the default 1.5× line-height padding so glyphs sit at
	     the visual center of the row, aligned with the icon box and toggle. -->
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
		<span class="text-base-mute font-mono text-[10px] leading-none">
			{__APP_VERSION__}
		</span>
		<button
			type="button"
			onclick={theme.toggle}
			aria-label="Cambiar tema"
			class="titlebar-brand-btn ml-1 flex h-5 w-5 shrink-0 items-center justify-center rounded transition-colors"
		>
			{#if theme.value === 'dark'}
				<IconSun size={12} />
			{:else}
				<IconMoon size={12} />
			{/if}
		</button>
	</div>

	<!-- Middle drag region: everything between brand and controls grabs the
	     window when you click-drag. -->
	<div class="h-full flex-1" data-tauri-drag-region></div>

	<!-- Window controls. -->
	<div class="flex h-full">
		<button
			type="button"
			onclick={minimize}
			aria-label="Minimizar"
			class="titlebar-btn flex h-full w-[46px] items-center justify-center transition-colors"
		>
			<svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden="true">
				<rect x="0" y="4.5" width="10" height="1" fill="currentColor" />
			</svg>
		</button>
		<button
			type="button"
			onclick={toggleMax}
			aria-label={maximized ? 'Restaurar' : 'Maximizar'}
			class="titlebar-btn flex h-full w-[46px] items-center justify-center transition-colors"
		>
			{#if maximized}
				<svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden="true">
					<rect
						x="2.5"
						y="0.5"
						width="7"
						height="7"
						stroke="currentColor"
						stroke-width="1"
						fill="none"
					/>
					<rect
						x="0.5"
						y="2.5"
						width="7"
						height="7"
						stroke="currentColor"
						stroke-width="1"
						fill="var(--bg-panel)"
					/>
				</svg>
			{:else}
				<svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden="true">
					<rect
						x="0.5"
						y="0.5"
						width="9"
						height="9"
						stroke="currentColor"
						stroke-width="1"
						fill="none"
					/>
				</svg>
			{/if}
		</button>
		<button
			type="button"
			onclick={close}
			aria-label="Cerrar"
			class="titlebar-btn titlebar-btn-close flex h-full w-[46px] items-center justify-center transition-colors"
		>
			<svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden="true">
				<path d="M0 0 L10 10 M10 0 L0 10" stroke="currentColor" stroke-width="1" />
			</svg>
		</button>
	</div>
</div>

<style>
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
	.titlebar-brand-btn {
		color: var(--text-mute);
	}
	.titlebar-brand-btn:hover {
		background: var(--hover);
		color: var(--text);
	}
</style>
