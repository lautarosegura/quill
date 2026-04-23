import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import * as api from '$lib/ipc/commands';
import { EVENTS } from '$lib/ipc/events';

let historyCount = $state<number | null>(null);
let unlisten: UnlistenFn | null = null;

export const counts = {
	get history() {
		return historyCount;
	},
	async refreshHistory() {
		try {
			historyCount = await api.countHistory();
		} catch {
			historyCount = null;
		}
	},
	/** Subscribe to backend `history_changed` events so the sidebar count
	 *  stays in sync whenever a new dictation lands (or a row is deleted). */
	async subscribe() {
		if (unlisten) return;
		unlisten = await listen(EVENTS.HISTORY_CHANGED, () => {
			void counts.refreshHistory();
		});
	},
	unsubscribe() {
		unlisten?.();
		unlisten = null;
	}
};
