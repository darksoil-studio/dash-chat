import path from 'path';
import { defineConfig } from 'vite';

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
	clearScreen: false,
	// 2. tauri expects a fixed port, fail if that port is not available
	server: {
		port: 1420,
		strictPort: true,
		host: '0.0.0.0',
		hmr: host
			? {
					protocol: 'ws',
					host: host,
					port: 1430,
				}
			: undefined,
	},
});
