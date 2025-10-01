import localIpAddress from 'local-ip-address';
import { defineConfig } from 'vite';

const host = localIpAddress();

export default defineConfig({
	clearScreen: false,
	// 2. tauri expects a fixed port, fail if that port is not available
	server: {
		port: 1420,
		strictPort: true,
		host: true,
		hmr: host
			? {
					protocol: 'ws',
					host: host,
					port: 1430,
				}
			: undefined,
	},
});
