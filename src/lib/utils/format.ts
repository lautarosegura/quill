/** Formats an ISO-8601 timestamp as "hace 4 min" / "hace 2 h" / absolute date for older. */
export function formatRelativeTime(isoDate: string): string {
	const then = new Date(isoDate);
	const now = new Date();
	const diffMs = now.getTime() - then.getTime();
	const diffSec = Math.floor(diffMs / 1000);
	const diffMin = Math.floor(diffSec / 60);
	const diffHour = Math.floor(diffMin / 60);
	const diffDay = Math.floor(diffHour / 24);

	if (diffSec < 10) return 'hace un instante';
	if (diffSec < 60) return `hace ${diffSec} s`;
	if (diffMin < 60) return `hace ${diffMin} min`;
	if (diffHour < 24) return `hace ${diffHour} h`;
	if (diffDay === 1) return 'ayer';
	if (diffDay < 7) return `hace ${diffDay} días`;
	return then.toLocaleDateString('es-AR', {
		day: '2-digit',
		month: 'short',
		year: 'numeric'
	});
}

/** Formats an ISO-8601 timestamp as a full absolute string for tooltips. */
export function formatAbsoluteTime(isoDate: string): string {
	const d = new Date(isoDate);
	return d.toLocaleString('es-AR', {
		dateStyle: 'medium',
		timeStyle: 'short'
	});
}

/** Formats milliseconds as "0:42" / "1:07" / "2:18". */
export function formatDuration(ms: number | null): string {
	if (ms == null) return '';
	const totalSec = Math.round(ms / 1000);
	const m = Math.floor(totalSec / 60);
	const s = totalSec % 60;
	return `${m}:${s.toString().padStart(2, '0')}`;
}

/** "12 kB" / "145 MB". */
export function formatBytes(bytes: number): string {
	if (bytes < 1024) return `${bytes} B`;
	if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
	if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(0)} MB`;
	return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

export type DayBucket = 'hoy' | 'ayer' | 'semana' | 'antes';

/** Buckets an ISO date into the group label used by Historial headers. */
export function dayBucket(isoDate: string): DayBucket {
	const then = new Date(isoDate);
	const now = new Date();
	if (then.toDateString() === now.toDateString()) return 'hoy';
	const yesterday = new Date(now);
	yesterday.setDate(yesterday.getDate() - 1);
	if (then.toDateString() === yesterday.toDateString()) return 'ayer';
	const sevenDaysAgo = new Date(now);
	sevenDaysAgo.setDate(sevenDaysAgo.getDate() - 7);
	if (then >= sevenDaysAgo) return 'semana';
	return 'antes';
}

export const DAY_BUCKET_LABEL: Record<DayBucket, string> = {
	hoy: 'HOY',
	ayer: 'AYER',
	semana: 'ESTA SEMANA',
	antes: 'ANTES'
};

import type { Keybind, Modifier } from '$lib/types';
import { platform, type Platform } from '$lib/stores/platform.svelte';

const MODIFIER_LABEL_WIN: Record<Modifier, string> = {
	ctrl: 'Ctrl',
	shift: 'Shift',
	alt: 'Alt',
	meta: 'Win'
};
const MODIFIER_LABEL_MAC: Record<Modifier, string> = {
	ctrl: '⌃',
	shift: '⇧',
	alt: '⌥',
	meta: '⌘'
};
const MODIFIER_LABEL_LINUX: Record<Modifier, string> = {
	ctrl: 'Ctrl',
	shift: 'Shift',
	alt: 'Alt',
	meta: 'Super'
};

function modifierMapFor(p: Platform): Record<Modifier, string> {
	if (p === 'macos') return MODIFIER_LABEL_MAC;
	if (p === 'linux') return MODIFIER_LABEL_LINUX;
	return MODIFIER_LABEL_WIN;
}

function formatKeyName(k: string): string {
	if (k === ' ' || k === 'Space' || k === 'Spacebar') return 'Space';
	if (k.length === 1) return k.toUpperCase();
	return k;
}

/** Expands a Keybind into an ordered list of labels for rendering, e.g.
 *  `["Ctrl", "Win", "Space"]` on Windows or `["⌃", "⌘"]` on macOS.
 *  Platform defaults to the auto-detected one from `platform.value`. */
export function keybindLabels(kb: Keybind, p: Platform = platform.value): string[] {
	const map = modifierMapFor(p);
	const parts = kb.modifiers.map((m) => map[m]);
	if (kb.key != null) parts.push(formatKeyName(kb.key));
	return parts;
}

/** Plain-text join, e.g. `"Ctrl + Win"` or `"⌃ + ⌘"`. */
export function keybindText(kb: Keybind, p: Platform = platform.value): string {
	return keybindLabels(kb, p).join(' + ');
}
