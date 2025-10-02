import type { Signal } from 'signal-polyfill';
import { effect } from 'signal-utils/subtle/microtask-effect';
import type { Readable } from 'svelte/store';

export function useSignal<T>(
	signal: Signal.State<T> | Signal.Computed<T>,
): Readable<T> {
	return {
		subscribe(fn: (value: T) => void) {
			return effect(() => {
				const value = signal.get();
				fn(value);
			});
		},
	};
}
