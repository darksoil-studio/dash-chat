export function isTauriEnv() {
	// eslint-disable-next-line
	return !!(window as any).__TAURI_INTERNALS__;
}

export const isMobile = /iPhone|iPad|iPod|Android/i.test(navigator.userAgent);
