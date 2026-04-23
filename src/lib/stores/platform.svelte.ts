import { invoke } from '@tauri-apps/api/core';

export type Platform = 'windows' | 'macos' | 'linux';

/** What Rust's `display_server::DisplayServer::detect()` returns. On Linux
 *  this is the critical distinction — `x11` means rdev/enigo work, `wayland`
 *  means we route through the XDG portal + clipboard-only paste fallback. */
export type DisplayServer = 'windows' | 'macos' | 'x11' | 'wayland';

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

// Display server is resolved lazily — it requires a Tauri invoke and the
// command is only available after the backend finishes setup. Consumers that
// need it call `initDisplayServer()` on mount (or read `platform.isWayland`,
// which is null-safe before resolution).
let _displayServer = $state<DisplayServer | null>(null);

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
	},
	get displayServer(): DisplayServer | null {
		return _displayServer;
	},
	/** True only once we've confirmed Wayland via the backend. Returns false
	 *  before `initDisplayServer()` has resolved — so callers that want to
	 *  show Wayland-specific UX should await the init first. */
	get isWayland(): boolean {
		return _displayServer === 'wayland';
	}
};

/** Resolves and caches the backend display server. Safe to call multiple
 *  times; only the first call hits the backend. */
export async function initDisplayServer(): Promise<DisplayServer> {
	if (_displayServer !== null) return _displayServer;
	try {
		_displayServer = await invoke<DisplayServer>('get_display_server');
	} catch {
		// Backend command unavailable — pick a reasonable fallback.
		_displayServer = _value === 'linux' ? 'x11' : _value;
	}
	return _displayServer;
}
