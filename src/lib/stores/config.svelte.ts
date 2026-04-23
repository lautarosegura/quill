import type { Config } from '$lib/types';
import * as api from '$lib/ipc/commands';

let configState = $state<Config | null>(null);
let loadingState = $state<boolean>(false);
let errorState = $state<string | null>(null);

let saveTimer: ReturnType<typeof setTimeout> | null = null;

export const config = {
	get value() {
		return configState;
	},
	get loading() {
		return loadingState;
	},
	get error() {
		return errorState;
	},

	async load() {
		loadingState = true;
		errorState = null;
		try {
			configState = await api.getConfig();
		} catch (e: unknown) {
			errorState = e instanceof Error ? e.message : String(e);
		} finally {
			loadingState = false;
		}
	},

	/** Immediate save — use for discrete UI events (radio, dropdown, toggle). */
	async set<K extends keyof Config>(key: K, value: Config[K]) {
		if (!configState) return;
		configState = { ...configState, [key]: value };
		try {
			await api.saveConfig(configState);
		} catch (e: unknown) {
			errorState = e instanceof Error ? e.message : String(e);
		}
	},

	/** Debounced save — use for continuous inputs (slider, textarea). */
	setDebounced<K extends keyof Config>(key: K, value: Config[K], ms = 500) {
		if (!configState) return;
		configState = { ...configState, [key]: value };
		if (saveTimer) clearTimeout(saveTimer);
		const next = configState;
		saveTimer = setTimeout(() => {
			api.saveConfig(next).catch((e) => {
				errorState = e instanceof Error ? e.message : String(e);
			});
		}, ms);
	},

	/** Full replace — used by the wizard. */
	async save(next: Config) {
		await api.saveConfig(next);
		configState = next;
	}
};
