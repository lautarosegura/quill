import { invoke } from '@tauri-apps/api/core';

export type Platform = 'windows' | 'macos' | 'linux';

/** What Rust's `display_server::DisplayServer::detect()` returns. On Linux
 *  this is the critical distinction — `x11` means rdev/enigo work, `wayland`
 *  means we route through the XDG portal + clipboard-only paste fallback. */
export type DisplayServer = 'windows' | 'macos' | 'x11' | 'wayland';

/** Mirrors `display_server::LinuxEnvironment` — the wizard uses this to decide
 *  whether to surface the "add yourself to the input group" setup card. */
export type LinuxEnvironment = {
	display_server: DisplayServer;
	desktop: string;
	gnome_version: number | null;
	kde_plasma_version: number | null;
};

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
let _linuxEnv = $state<LinuxEnvironment | null>(null);

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
	},
	get linuxEnvironment(): LinuxEnvironment | null {
		return _linuxEnv;
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

/** Resolves and caches the user's Linux compositor + GNOME version. Returns
 *  null on non-Linux platforms (the backend command itself returns null
 *  there). Safe to call multiple times. */
export async function initLinuxEnvironment(): Promise<LinuxEnvironment | null> {
	if (_linuxEnv !== null) return _linuxEnv;
	if (_value !== 'linux') return null;
	try {
		_linuxEnv = await invoke<LinuxEnvironment | null>('get_linux_environment');
	} catch {
		_linuxEnv = null;
	}
	return _linuxEnv;
}

/** True iff the current Linux compositor lacks portal-based hotkey support
 *  (GNOME pre-48, KDE Plasma 5, Sway, Hyprland, wlroots-based) — i.e. the
 *  wizard should surface the "add yourself to the input group" card.
 *  Returns false on X11, on Windows / macOS, on GNOME 48+, on confirmed
 *  KDE Plasma 6+, and before `initLinuxEnvironment()` has resolved. */
export function needsInputGroupSetup(env: LinuxEnvironment | null): boolean {
	if (!env) return false;
	if (env.display_server !== 'wayland') return false;
	const desktop = env.desktop.toLowerCase();
	// GNOME 48+ has the GlobalShortcuts portal — zero config.
	if (desktop === 'gnome') return (env.gnome_version ?? 0) < 48;
	// KDE: only Plasma 6+ ships the portal. Plasma 5 (still on LTS distros
	// like Ubuntu 22.04) needs the input-group fallback. If the version
	// can't be confirmed, fall through and show the card — false negatives
	// are worse than false positives here.
	if (desktop === 'kde') {
		return (env.kde_plasma_version ?? 0) < 6;
	}
	// Everything else (Sway, Hyprland, wlroots, Cinnamon, MATE…) needs
	// the rdev evdev fallback, which requires the input group.
	return true;
}
