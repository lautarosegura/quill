<script lang="ts">
	import { fade } from 'svelte/transition';
	import Waveform from './Waveform.svelte';
	import DotSpinner from './DotSpinner.svelte';

	type State = 'recording' | 'transcribing' | 'error' | 'cancelled' | 'clipboard-only';

	interface Props {
		state: State;
		message?: string;
		onClose?: () => void;
	}
	let { state, message, onClose }: Props = $props();

	const BORDER: Record<State, string> = {
		recording: '#EF4444',
		transcribing: '#F59E0B',
		error: '#F43F5E',
		// Cancelled uses a muted gray so it reads as "neutral, not-an-error".
		cancelled: 'oklch(0.55 0.01 260)',
		// ClipboardOnly is a success outcome on Wayland — use a friendly
		// blue so it reads as "ready to paste" not "something went wrong".
		'clipboard-only': '#3B82F6'
	};

	// Content transition: short crossfade when state changes so the icon/waveform
	// swap feels continuous with the border-color morph (240ms) on the outer shell.
	const CROSSFADE_MS = 180;
</script>

<!-- Outer shell persists across state changes; only the border-color morphs
     (via CSS transition) and the inner content crossfades (via Svelte). The
     pill slides-up + fades in on mount thanks to the .pill-enter animation. -->
<div
	class="pill-enter relative flex items-center px-4"
	style="
		width: 220px; height: 48px; border-radius: 24px;
		background: oklch(0.12 0.008 260 / 0.95);
		backdrop-filter: blur(16px) saturate(140%);
		-webkit-backdrop-filter: blur(16px) saturate(140%);
		box-shadow: 0 10px 40px -8px oklch(0 0 0 / 0.5), 0 0 0 0.5px oklch(0 0 0 / 0.5);
		border: 1px solid {BORDER[state]};
		color: white;
		transition: border-color 240ms ease, box-shadow 240ms ease;
	"
>
	{#if state === 'recording'}
		<div
			class="flex flex-1 items-center gap-2.5"
			in:fade={{ duration: CROSSFADE_MS }}
			out:fade={{ duration: CROSSFADE_MS }}
		>
			<span class="relative flex h-2.5 w-2.5 shrink-0 items-center justify-center">
				<span
					class="absolute inset-0 rounded-full"
					style="background: {BORDER.recording}; animation: pulseRing 1.4s ease-out infinite"
				></span>
				<span
					class="relative h-2 w-2 rounded-full"
					style="background: {BORDER.recording}; animation: pulseRec 1.4s ease-in-out infinite"
				></span>
			</span>
			<Waveform active color="white" />
			<span class="flex-1 text-right text-[14px] font-medium tracking-tight">Grabando</span>
		</div>
	{:else if state === 'transcribing'}
		<div
			class="flex flex-1 items-center gap-3"
			in:fade={{ duration: CROSSFADE_MS }}
			out:fade={{ duration: CROSSFADE_MS }}
		>
			<DotSpinner color={BORDER.transcribing} />
			<span class="shimmer-text flex-1 text-[14px] font-medium tracking-tight">Transcribiendo…</span>
		</div>
	{:else if state === 'cancelled'}
		<div
			class="flex flex-1 items-center gap-2.5"
			in:fade={{ duration: CROSSFADE_MS }}
			out:fade={{ duration: CROSSFADE_MS }}
		>
			<span
				class="flex h-[18px] w-[18px] shrink-0 items-center justify-center rounded-full"
				style="background: oklch(1 0 0 / 0.1); color: {BORDER.cancelled}"
			>
				<svg
					width="11"
					height="11"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2.5"
					stroke-linecap="round"
				>
					<path d="M6 6l12 12M18 6L6 18" />
				</svg>
			</span>
			<span class="flex-1 truncate text-[14px] font-medium tracking-tight">
				Transcripción cancelada
			</span>
		</div>
	{:else if state === 'clipboard-only'}
		<div
			class="flex flex-1 items-center gap-2.5"
			in:fade={{ duration: CROSSFADE_MS }}
			out:fade={{ duration: CROSSFADE_MS }}
		>
			<span
				class="flex h-[18px] w-[18px] shrink-0 items-center justify-center rounded-full"
				style="background: oklch(1 0 0 / 0.12); color: {BORDER['clipboard-only']}"
			>
				<!-- Clipboard icon -->
				<svg
					width="11"
					height="11"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<rect x="8" y="2" width="8" height="4" rx="1" />
					<path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" />
				</svg>
			</span>
			<span class="flex-1 truncate text-[14px] font-medium tracking-tight">
				Ctrl+V para pegar
			</span>
		</div>
	{:else if state === 'error'}
		<div
			class="flex flex-1 items-center gap-2.5"
			in:fade={{ duration: CROSSFADE_MS }}
			out:fade={{ duration: CROSSFADE_MS }}
		>
			<span
				class="flex h-[18px] w-[18px] shrink-0 items-center justify-center rounded-full"
				style="background: oklch(0.55 0.22 15 / 0.2); color: {BORDER.error}"
			>
				<svg
					width="11"
					height="11"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2.5"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<path d="M12 9v4" />
					<path d="M12 17h.01" />
					<path d="M10.3 3.9 2.4 18a2 2 0 0 0 1.7 3h15.8a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z" />
				</svg>
			</span>
			<span class="flex-1 truncate text-[14px] font-medium tracking-tight">
				{message ?? 'Error'}
			</span>
			{#if onClose}
				<button
					onclick={onClose}
					class="flex h-5 w-5 shrink-0 items-center justify-center rounded-full transition-colors hover:bg-[oklch(1_0_0_/_0.16)]"
					style="color: oklch(0.75 0.01 260); background: oklch(1 0 0 / 0.08)"
					aria-label="Cerrar"
				>
					<svg
						width="9"
						height="9"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2.5"
						stroke-linecap="round"
					>
						<path d="M6 6l12 12M18 6L6 18" />
					</svg>
				</button>
			{/if}
		</div>
	{/if}
</div>

<style>
	/* Pill entrance: slide-up + fade + subtle scale, matching the Claude
	   Design spec (200ms, cubic-bezier(0.2, 0.8, 0.2, 1)). */
	@keyframes pillEnter {
		from {
			opacity: 0;
			transform: translateY(12px) scale(0.96);
		}
		to {
			opacity: 1;
			transform: none;
		}
	}
	.pill-enter {
		animation: pillEnter 200ms cubic-bezier(0.2, 0.8, 0.2, 1);
	}

	/* Let the Svelte `in:` / `out:` transitions overlap cleanly — absolute
	   positioning on the inner content blocks so they occupy the same slot
	   during the crossfade. */
	.pill-enter > :global(div) {
		position: absolute;
		inset: 0;
		padding: 0 16px;
	}
</style>
