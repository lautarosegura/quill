export type Platform = 'windows' | 'macos' | 'linux';

/**
 * Detect platform from the webview's user agent. Runs synchronously so modules
 * that import this store can read `platform.value` immediately without waiting
 * on an async Tauri command. Accurate enough for labeling modifier keys
 * (Ctrl/Win vs ⌃/⌘) — the only place we need this distinction today.
 */
function detect(): Platform {
	if (typeof navigator === 'undefined') return 'windows';
	const ua = navigator.userAgent;
	if (/Macintosh|Mac OS X/i.test(ua)) return 'macos';
	if (/Linux/i.test(ua)) return 'linux';
	return 'windows';
}

const _value: Platform = detect();

export const platform = {
	get value(): Platform {
		return _value;
	},
	get isMac(): boolean {
		return _value === 'macos';
	},
	get isWindows(): boolean {
		return _value === 'windows';
	},
	get isLinux(): boolean {
		return _value === 'linux';
	}
};
