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
	| { state: 'error'; message: string };
