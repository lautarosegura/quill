<script lang="ts">
	type State = 'idle' | 'recording' | 'transcribing' | 'injecting' | 'error' | 'cancelled';

	interface Props {
		state: State;
	}
	let { state }: Props = $props();

	const CFG: Record<State, { dot: string; pulse: boolean; spin: boolean }> = {
		idle: { dot: 'var(--text-mute)', pulse: false, spin: false },
		recording: { dot: 'var(--accent)', pulse: true, spin: false },
		transcribing: { dot: 'oklch(0.78 0.14 75)', pulse: false, spin: true },
		injecting: { dot: 'oklch(0.72 0.15 145)', pulse: false, spin: false },
		error: { dot: 'oklch(0.72 0.18 25)', pulse: false, spin: false },
		cancelled: { dot: 'var(--text-mute)', pulse: false, spin: false }
	};
</script>

<span class="relative inline-flex h-2 w-2 items-center justify-center">
	{#if CFG[state].pulse}
		<span
			class="rec-ring absolute inset-0 rounded-full"
			style="background: {CFG[state].dot}"
		></span>
	{/if}
	<span
		class="relative h-2 w-2 rounded-full {CFG[state].pulse ? 'rec-dot' : ''}"
		style="background: {CFG[state].dot}; {CFG[state].spin
			? 'animation: pulse-rec 1.2s ease-in-out infinite'
			: ''}"
	></span>
</span>
