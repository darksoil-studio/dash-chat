import { writable } from 'svelte/store';

const SPLASHSCREEN_ITEM = 'splashscreendismissed';

function buildSplashscreenStore() {
	const store = writable(hasSplashscreenBeenDismissed());

	return {
		subscribe: store.subscribe,
		dismiss: () => {
			dismissSplashscreen();
			store.set(true);
		},
	};
}

export const splashscreenDismissed = buildSplashscreenStore();

function hasSplashscreenBeenDismissed(): boolean {
	return !!localStorage.getItem(SPLASHSCREEN_ITEM);
}

function dismissSplashscreen() {
	localStorage.setItem(SPLASHSCREEN_ITEM, 'true');
}
