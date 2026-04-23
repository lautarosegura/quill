import { listen } from '@tauri-apps/api/event';
import { EVENTS, type TranscriptionState } from '$lib/ipc/events';

let currentState = $state<TranscriptionState>({ state: 'idle' });
let subscribed = false;
let unlisten: (() => void) | null = null;

export const status = {
	get value() {
		return currentState;
	}
};

export async function subscribeToStatus() {
	if (subscribed) return;
	subscribed = true;
	unlisten = await listen<TranscriptionState>(
		EVENTS.TRANSCRIPTION_STATE_CHANGED,
		(event) => {
			currentState = event.payload;
		}
	);
}

export function unsubscribeFromStatus() {
	subscribed = false;
	unlisten?.();
	unlisten = null;
}
