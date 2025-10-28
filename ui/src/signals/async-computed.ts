import { Signal } from 'signal-polyfill';
import { AsyncComputed as NativeAsyncComputed } from 'signal-utils/async-computed';
import { effect } from 'signal-utils/subtle/microtask-effect';

import type { AsyncRelay, AsyncResult, UnsubscribeFn } from './relay';

export type AnySignal<T> = Signal.State<T> | Signal.Computed<T>;

export class SvelteState<T> extends Signal.State<T> {
	subscribe(fn: (value: T) => void) {
		return effect(() => {
			const value = this.get();
			fn(value);
		});
	}
}

// export function load<T>(signal: AsyncSignal<T>): Promise<T> {
// 	return new Promise((resolve, reject) => {
// 		// const state = new SvelteState<T | undefined>(undefined, {
// 		// 	[Signal.subtle.unwatched]: () => {
// 		// 		unsubs();
// 		// 	},
// 		// });
// 		const unsubs = effect(() => {
// 			const value = signal.get();
// 			if (value.status === 'completed') {
// 				resolve(value.value);
// 			} else if (value.status === 'error') {
// 				reject(value.error);
// 				unsubs();
// 			}
// 		});
// 	});
// }

// export function load<T>(signal: AsyncSignal<T>): Promise<AnySignal<T>> {
// 	return new Promise((resolve, reject) => {
// 		const state = new SvelteState<T | undefined>(undefined, {
// 			[Signal.subtle.unwatched]: () => {
// 				unsubs();
// 			},
// 		});
// 		const unsubs = effect(() => {
// 			const value = signal.get();
// 			if (value.status === 'completed') {
// 				state.set(value.value);
// 				resolve(state as Signal.State<T>);
// 			} else if (value.status === 'error') {
// 				reject(value.error);
// 				unsubs();
// 			}
// 		});
// 	});
// }

export class AsyncComputed<T> extends NativeAsyncComputed<T> {
	subscribe(fn: (value: Promise<T>) => void): UnsubscribeFn {
		fn(Promise.resolve({} as any));
		return () => {};
		// console.log('aaa');
		// return effect(() => {
		// 	const status = this.status;
		// 	console.log('aaa2', status);
		// 	switch (status) {
		// 		case 'initial':
		// 		case 'pending':
		// 			fn(new Promise(r => {}));
		// 			break;
		// 		case 'error':
		// 			const error = this.error;
		// 			fn(Promise.reject(error));
		// 			break;
		// 		case 'complete':
		// 			const value = this.get();
		// 			fn(Promise.resolve(value!));
		// 			break;
		// 	}
		// });
	}

	// load() {
	// 	return load(this);
	// }
}

export type AsyncSignal<T> = AsyncComputed<T> | AsyncRelay<T>;
