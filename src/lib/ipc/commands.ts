import { invoke } from '@tauri-apps/api/core';
import type { Config } from '$lib/types';

// Config ---------------------------------------------------------------------

export async function getConfig(): Promise<Config> {
	return invoke<Config>('get_config');
}

export async function saveConfig(config: Config): Promise<void> {
	return invoke<void>('save_config', { config });
}

export async function isFirstRun(): Promise<boolean> {
	return invoke<boolean>('is_first_run');
}

export async function getDefaultHotkey(): Promise<import('$lib/types').Keybind> {
	return invoke<import('$lib/types').Keybind>('get_default_hotkey');
}

// Groq -----------------------------------------------------------------------

export async function getGroqKeyMasked(): Promise<string | null> {
	return invoke<string | null>('get_groq_key_masked');
}

export async function setGroqKey(key: string): Promise<void> {
	return invoke<void>('set_groq_key', { key });
}

export async function deleteGroqKey(): Promise<void> {
	return invoke<void>('delete_groq_key');
}

export interface GroqTestResult {
	valid: boolean;
	message: string;
}

export async function testGroqKey(key: string): Promise<GroqTestResult> {
	return invoke<GroqTestResult>('test_groq_key', { key });
}

export type GroqModelKind = 'verified' | 'catalog_only' | 'new_unknown';

export interface GroqModelEntry {
	name: string;
	display_name: string;
	description: string;
	cost_per_hour_usd: number | null;
	languages: string[];
	kind: GroqModelKind;
}

export interface GroqModelsResult {
	live_check_succeeded: boolean;
	error: string | null;
	models: GroqModelEntry[];
}

export async function listGroqModels(): Promise<GroqModelsResult> {
	return invoke<GroqModelsResult>('list_groq_models');
}

export async function setGroqModel(model: string): Promise<void> {
	return invoke<void>('set_groq_model', { model });
}

export async function refreshGroqEngine(): Promise<void> {
	return invoke<void>('refresh_groq_engine');
}

// Devices --------------------------------------------------------------------

export interface MicDevice {
	name: string;
	is_default: boolean;
}

export async function listMicDevices(): Promise<MicDevice[]> {
	return invoke<MicDevice[]>('list_mic_devices');
}

export async function startMicTest(device: string | null): Promise<void> {
	return invoke<void>('start_mic_test', { device });
}

export async function stopMicTest(): Promise<void> {
	return invoke<void>('stop_mic_test');
}

export interface LocalModel {
	name: string;
	size_bytes: number;
}

export async function listLocalModels(): Promise<LocalModel[]> {
	return invoke<LocalModel[]>('list_local_models');
}

// Dictation (debug) ----------------------------------------------------------

export async function triggerTestDictation(): Promise<void> {
	return invoke<void>('trigger_test_dictation');
}

// History --------------------------------------------------------------------

export interface HistoryEntry {
	id: number;
	created_at: string;
	engine: 'local' | 'groq';
	language: string;
	model: string | null;
	duration_ms: number | null;
	latency_ms: number | null;
	text: string;
	status: 'success' | 'failed';
	failure_reason: string | null;
	failed_wav_path: string | null;
	/** Foreground window title at the moment the user pressed the hotkey.
	 *  Windows only today; null elsewhere. */
	source_app: string | null;
}

export async function listHistory(limit: number, offset: number): Promise<HistoryEntry[]> {
	return invoke<HistoryEntry[]>('list_history', { limit, offset });
}

export async function searchHistory(query: string, limit: number): Promise<HistoryEntry[]> {
	return invoke<HistoryEntry[]>('search_history', { query, limit });
}

export async function deleteHistoryEntry(id: number): Promise<void> {
	return invoke<void>('delete_history_entry', { id });
}

export async function clearAllHistory(): Promise<void> {
	return invoke<void>('clear_all_history');
}

export async function countHistory(): Promise<number> {
	return invoke<number>('count_history');
}

export async function reinjectHistoryEntry(id: number): Promise<void> {
	return invoke<void>('reinject_history_entry', { id });
}

export async function retryHistoryEntry(id: number): Promise<void> {
	return invoke<void>('retry_history_entry', { id });
}

// Hardware ------------------------------------------------------------------

export interface HardwareProfile {
	os: string;
	arch: string;
	apple_silicon: boolean;
	ram_gb: number;
	cpu_cores: number;
	recommended_model: string;
	recommended_rationale: string;
}

export async function detectHardware(): Promise<HardwareProfile> {
	return invoke<HardwareProfile>('detect_hardware');
}

// Models (known + download) --------------------------------------------------

export interface ModelEntry {
	name: string;
	display_name: string;
	size_bytes: number;
	description: string;
	installed: boolean;
	installed_size_bytes: number | null;
}

export async function listKnownModels(): Promise<ModelEntry[]> {
	return invoke<ModelEntry[]>('list_known_models');
}

export async function downloadModel(name: string): Promise<void> {
	return invoke<void>('download_model', { name });
}

export async function deleteModel(name: string): Promise<void> {
	return invoke<void>('delete_model', { name });
}

export interface ModelDownloadProgress {
	name: string;
	downloaded: number;
	total: number;
}

export interface ModelDownloadComplete {
	name: string;
}

export interface ModelDownloadError {
	name: string;
	message: string;
}

// Usage ---------------------------------------------------------------------

export interface DailyCount {
	date: string; // YYYY-MM-DD
	count: number;
}

export interface UsageStats {
	month: string; // YYYY-MM
	total_transcriptions: number;
	local_transcriptions: number;
	groq_transcriptions: number;
	total_audio_seconds: number;
	groq_audio_seconds: number;
	estimated_groq_cost_usd: number;
	daily_counts: DailyCount[];
}

export async function getUsageStats(): Promise<UsageStats> {
	return invoke<UsageStats>('get_usage_stats');
}

// Wizard --------------------------------------------------------------------

export async function finishWizard(): Promise<void> {
	return invoke<void>('finish_wizard');
}
