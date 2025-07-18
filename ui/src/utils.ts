import {
	attachConsole,
	error,
	info,
	trace,
	warn,
} from '@tauri-apps/plugin-log';

export const sleep = (ms: number) =>
	new Promise(r => setTimeout(() => r(undefined), ms));

export function getOS() {
	var userAgent = window.navigator.userAgent,
		platform = window.navigator.platform,
		macosPlatforms = ['Macintosh', 'MacIntel', 'MacPPC', 'Mac68K'],
		windowsPlatforms = ['Win32', 'Win64', 'Windows', 'WinCE'],
		iosPlatforms = ['iPhone', 'iPad', 'iPod'],
		os = null;

	if (macosPlatforms.indexOf(platform) !== -1) {
		os = 'Mac OS';
	} else if (iosPlatforms.indexOf(platform) !== -1) {
		os = 'iOS';
	} else if (windowsPlatforms.indexOf(platform) !== -1) {
		os = 'Windows';
	} else if (/Android/.test(userAgent)) {
		os = 'Android';
	} else if (!os && /Linux/.test(platform)) {
		os = 'Linux';
	}

	return os;
}
export function isMobileOs() {
	return getOS() === 'Android' || getOS() === 'iOS';
}

export function onNotificationClicked(
	handler: (notification: Notification) => void,
) {}

export async function withRetries<T>(task: () => Promise<T>, retries = 10) {
	try {
		const r = await task();
		return r;
	} catch (e) {
		if (retries - 1 == 0) throw new Error(`Timeout. Last error: ${e}`);
		await sleep(1000);
		return withRetries(task, retries - 1);
	}
}

export function connectConsoleToLogs() {
	const l = console.log;
	console.log = d => {
		info(d);
		l(d);
	};
	const w = console.warn;
	console.log = d => {
		warn(d);
		w(d);
	};
	const e = console.error;
	console.log = d => {
		error(d);
		e(d);
	};
}
