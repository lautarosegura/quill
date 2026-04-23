import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import tailwindcss from '@tailwindcss/vite';
import { readFileSync } from 'node:fs';

const pkg = JSON.parse(readFileSync('./package.json', 'utf8'));

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	// Tauri works best with a fixed dev port.
	server: {
		port: 5173,
		strictPort: true,
		host: '127.0.0.1'
	},
	// Prevent vite from obscuring rust errors in dev.
	clearScreen: false,
	// tauri expects this
	envPrefix: ['VITE_', 'TAURI_'],
	define: {
		__APP_VERSION__: JSON.stringify(pkg.version)
	}
});
