export const EVENTS = {
	TRANSCRIPTION_STATE_CHANGED: 'transcription_state_changed',
	CONFIG_CHANGED: 'config_changed',
	PERMISSION_STATE_CHANGED: 'permission_state_changed',
	/** Fired by Rust after every successful or failed history-row insert so
	 *  the Historial view (and sidebar count) refresh without polling. */
	HISTORY_CHANGED: 'history_changed'
} as const;

export type EventName = (typeof EVENTS)[keyof typeof EVENTS];

export type TranscriptionState =
	| { state: 'idle' }
	| { state: 'recording' }
	| { state: 'transcribing' }
	| { state: 'injecting' }
	/** User aborted a locked/in-progress recording via Escape. Shown briefly,
	 *  then auto-transitions to Idle. */
	| { state: 'cancelled' }
	/** Wayland compositor denied programmatic input injection; the text was
	 *  copied to the clipboard and the user must press Ctrl+V to paste.
	 *  Shown briefly, then auto-transitions to Idle. */
	| { state: 'clipboard-only'; text_len: number }
	| { state: 'error'; message: string };
