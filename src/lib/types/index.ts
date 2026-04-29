export type Language = 'es' | 'en';
export type Engine = 'local' | 'groq';
/** Cloud LLM providers used by the optional post-transcription polish stage.
 *  Each one has its own keychain slot and its own preferred model. */
export type LlmProvider = 'groq' | 'anthropic' | 'openai';
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

export interface Substitution {
	from: string;
	to: string;
	case_sensitive: boolean;
}

export interface PromptPreset {
	id: string;
	name: string;
	prompt: string;
	builtin: boolean;
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
	substitutions: Substitution[];
	presets: PromptPreset[];
	active_preset_id: string | null;
	/** Master toggle. Off by default — opt-in. */
	llm_polish_enabled: boolean;
	/** Active provider used when polish runs. Each provider has its own
	 *  keychain key and its own model preference. */
	llm_polish_provider: LlmProvider;
	/** Per-provider chosen model id. Allows the user to switch providers
	 *  without losing each one's preferred model. */
	llm_polish_models: Partial<Record<LlmProvider, string>>;
	/** User-editable system prompt sent to the LLM. */
	llm_polish_system_prompt: string;
	/** Safety cap; texts longer than this skip the polish call. */
	llm_polish_max_input_chars: number;
}

export interface LlmModelInfo {
	provider: LlmProvider;
	id: string;
	display_name: string;
	blurb: string;
	recommended: boolean;
}

export interface LlmKeyTestResult {
	valid: boolean;
	message: string;
}

export interface PolishPreviewResult {
	original: string;
	polished: string;
	latency_ms: number;
	model: string;
	input_tokens: number | null;
	output_tokens: number | null;
}

export interface SerializableError {
	code: string;
	message: string;
}
