<script lang="ts">
	import type { Keybind, Modifier } from '$lib/types';
	import { platform } from '$lib/stores/platform.svelte';
	import KeyCap from './KeyCap.svelte';
	import Button from './Button.svelte';

	interface Props {
		value: Keybind;
		onChange: (next: Keybind) => void;
		/** Optional: shows a "Restaurar default" button. Called with the platform default. */
		onResetDefault?: () => void;
	}
	let { value, onChange, onResetDefault }: Props = $props();

	let capturing = $state(false);
	/** Modifiers held while capturing — used to save modifier-only chords on keyup. */
	let heldMods = $state<Set<Modifier>>(new Set());

	// Platform-appropriate modifier labels. Windows users see "Ctrl/Win",
	// Mac users see "⌃/⌘", Linux sees "Ctrl/Super".
	const MODIFIER_LABEL: Record<Modifier, string> = platform.isMac
		? { ctrl: '⌃', shift: '⇧', alt: '⌥', meta: '⌘' }
		: platform.isLinux
			? { ctrl: 'Ctrl', shift: 'Shift', alt: 'Alt', meta: 'Super' }
			: { ctrl: 'Ctrl', shift: 'Shift', alt: 'Alt', meta: 'Win' };

	function formatKey(k: string): string {
		if (k === ' ' || k === 'Space' || k === 'Spacebar') return 'Space';
		if (k.length === 1) return k.toUpperCase();
		return k;
	}

	function isModifierKey(k: string): boolean {
		return k === 'Control' || k === 'Shift' || k === 'Alt' || k === 'Meta';
	}

	function modifiersFromEvent(e: KeyboardEvent): Modifier[] {
		const m: Modifier[] = [];
		if (e.ctrlKey) m.push('ctrl');
		if (e.shiftKey) m.push('shift');
		if (e.altKey) m.push('alt');
		if (e.metaKey) m.push('meta');
		return m;
	}

	function handleKeyDown(e: KeyboardEvent) {
		if (!capturing) return;

		e.preventDefault();
		e.stopPropagation();

		// Escape with no modifiers cancels capture.
		if (e.key === 'Escape' && !e.ctrlKey && !e.shiftKey && !e.altKey && !e.metaKey) {
			capturing = false;
			heldMods = new Set();
			return;
		}

		if (isModifierKey(e.key)) {
			// Remember what's currently held so we can save a modifier-only chord
			// on release.
			heldMods = new Set(modifiersFromEvent(e));
			return;
		}

		// A non-modifier was pressed — save as classic trigger chord.
		const modifiers = modifiersFromEvent(e);
		if (modifiers.length === 0) return;

		onChange({ modifiers, key: formatKey(e.key) });
		capturing = false;
		heldMods = new Set();
	}

	function handleKeyUp(e: KeyboardEvent) {
		if (!capturing) return;
		if (!isModifierKey(e.key)) return;

		// Only save the modifier-only chord when the LAST modifier goes up
		// (otherwise every modifier release would clobber the combo).
		const stillHeld = modifiersFromEvent(e);
		if (stillHeld.length > 0) {
			heldMods = new Set(stillHeld);
			return;
		}

		// All modifiers released. If we captured a meaningful chord, save it.
		const mods = Array.from(heldMods);
		heldMods = new Set();
		if (mods.length >= 2) {
			onChange({ modifiers: mods, key: null });
			capturing = false;
		}
		// A single modifier tap isn't a valid hotkey — ignore and keep capturing.
	}

	$effect(() => {
		if (!capturing) return;
		window.addEventListener('keydown', handleKeyDown, { capture: true });
		window.addEventListener('keyup', handleKeyUp, { capture: true });
		return () => {
			window.removeEventListener('keydown', handleKeyDown, { capture: true });
			window.removeEventListener('keyup', handleKeyUp, { capture: true });
		};
	});
</script>

{#if capturing}
	<div
		class="text-base-dim flex items-center rounded-md px-3 py-2 text-[12px]"
		style="border: 1px dashed var(--border-strong)"
	>
		Apretá una combinación... (soltá los modificadores para guardar un chord sin tecla extra)
	</div>
{:else}
	<div class="flex flex-wrap items-center gap-2">
		<div class="flex items-center gap-1">
			{#each value.modifiers as mod, i (mod)}
				<KeyCap size="big">{MODIFIER_LABEL[mod]}</KeyCap>
				{#if value.key != null || i < value.modifiers.length - 1}
					<span class="text-base-mute text-[12px]">+</span>
				{/if}
			{/each}
			{#if value.key != null}
				<KeyCap size="big">{formatKey(value.key)}</KeyCap>
			{/if}
		</div>
		<Button size="sm" variant="secondary" onclick={() => (capturing = true)}>Cambiar</Button>
		{#if onResetDefault}
			<Button size="sm" variant="ghost" onclick={onResetDefault}>Restaurar default</Button>
		{/if}
	</div>
{/if}
