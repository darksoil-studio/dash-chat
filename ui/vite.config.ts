import { paraglideVitePlugin } from '@inlang/paraglide-js';
import { sveltekit } from '@sveltejs/kit/vite';
import localIpAddress from 'local-ip-address';
import { defineConfig } from 'vite';

// @ts-expect-error process is a nodejs global
// const host = process.env.TAURI_DEV_HOST;
const host = localIpAddress();

// https://vite.dev/config/
export default defineConfig(async () => ({
	optimizeDeps: { exclude: ['../packages/dash-chat-stores'] },
	plugins: [
		sveltekit(),
		paraglideVitePlugin({
			project: './project.inlang',
			outdir: './src/lib/paraglide',
		}),
	],
	// Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
	//
	// 1. prevent Vite from obscuring rust errors
	clearScreen: false,
	// 2. tauri expects a fixed port, fail if that port is not available
	server: {
		port: 1420,
		strictPort: true,
		host: true,
		hmr: host ? { protocol: 'ws', host, port: 1421 } : undefined,
		watch: {
			// 3. tell Vite to ignore watching `src-tauri`
			ignored: ['**/src-tauri/**'],
		},
	},
}));
