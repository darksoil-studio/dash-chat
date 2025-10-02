import { Signal } from 'signal-polyfill';
import { effect } from 'signal-utils/subtle/microtask-effect';

import type { AsyncRelay, AsyncResult } from './relay';

export type AnySignal<T> =
	| Signal.State<T>
	| (Signal.Computed<T> & { subscribe: (value: T) => void });

export class SvelteState<T> extends Signal.State<T> {
	subscribe(fn: (value: T) => void) {
		return effect(() => {
			const value = this.get();
			fn(value);
		});
	}
}

export function load<T>(signal: AsyncSignal<T>): Promise<AnySignal<T>> {
	return new Promise((resolve, reject) => {
		const state = new SvelteState<T | undefined>(undefined, {
			[Signal.subtle.unwatched]: () => {
				unsubs();
			},
		});
		const unsubs = effect(() => {
			const value = signal.get();
			if (value.status === 'completed') {
				state.set(value.value);
				resolve(state as Signal.State<T>);
			} else if (value.status === 'error') {
				reject(value.error);
				unsubs();
			}
		});
	});
}

export class AsyncComputed<T> extends Signal.Computed<AsyncResult<T>> {
	load(): Promise<AnySignal<T>> {
		return load(this);
	}
	subscribe(fn: (value: T) => void) {
		return effect(() => {
			const value = this.get();
			fn(value);
		});
	}
}

export type AsyncSignal<T> = AsyncComputed<T> | AsyncRelay<T>;
