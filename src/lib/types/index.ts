export type Language = 'es' | 'en';
export type Engine = 'local' | 'groq';
export type OverlayPosition =
	| 'bottom-center'
	| 'bottom-left'
	| 'bottom-right'
	| 'top-center'
	| 'top-left'
	| 'top-right';

export type Modifier = 'ctrl' | 'shift' | 'alt' | 'meta';

export interface Keybind {
	modifiers: Modifier[];
	/** `null` means the chord is modifier-only (e.g. Ctrl + Win on Windows). */
	key: string | null;
}

export interface Config {
	language: Language;
	engine: Engine;
	local_model_name: string;
	groq_model: string;
	hotkey: Keybind;
	language_cycle_hotkey: Keybind | null;
	mic_device: string | null;
	overlay_position: OverlayPosition;
	max_duration_secs: number;
	min_duration_ms: number;
	start_on_boot: boolean;
	sounds_enabled: boolean;
	vocabulary: string;
	monthly_cost_alert_usd: number | null;
	wizard_version: number;
}

export interface SerializableError {
	code: string;
	message: string;
}
