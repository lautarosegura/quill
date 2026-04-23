import type { Keybind, Language } from '$lib/types';

export type EngineChoice = 'local' | 'groq' | 'both';

export interface WizardDraft {
	step: number; // 1..5
	engineChoice: EngineChoice | null;
	groqKey: string;
	groqKeyTested: boolean;
	localModel: string;
	localModelDownloaded: boolean;
	language: Language;
	hotkey: Keybind;
}

// Placeholder seed. The wizard's mount hook calls `getConfig()` and patches
// the real platform-specific default from Rust (Ctrl + Win on Windows,
// Ctrl + Shift + Space elsewhere).
const DEFAULT_HOTKEY: Keybind = { modifiers: ['ctrl', 'shift'], key: 'Space' };

let state = $state<WizardDraft>({
	step: 1,
	engineChoice: null,
	groqKey: '',
	groqKeyTested: false,
	localModel: 'ggml-base',
	localModelDownloaded: false,
	language: 'es',
	hotkey: DEFAULT_HOTKEY
});

export const wizard = {
	get draft() {
		return state;
	},
	setStep(step: number) {
		state.step = Math.max(1, Math.min(5, step));
	},
	next() {
		state.step = Math.min(5, state.step + 1);
	},
	prev() {
		state.step = Math.max(1, state.step - 1);
	},
	patch<K extends keyof WizardDraft>(key: K, value: WizardDraft[K]) {
		(state as WizardDraft)[key] = value;
	}
};
