import {
	type ReactiveFn,
	ReactivePromise,
	reactive,
	signal,
	watcher,
} from 'signalium';
import { type Readable } from 'svelte/store';

export function useSignal<T, Args extends unknown[]>(
	v: ReactiveFn<T, Args>,
	...args: Args
): Readable<T> {
	const w = watcher(() => {
		const value = v(...args);

		if (value instanceof ReactivePromise && value.value) return value.value;
		if (value instanceof Promise) {
			const s = signal(value);

			value.then(v => (s.value = v));

			return s.value;
		}
		return value;
	});
	return {
		subscribe: set => {
			const unsubs = w.addListener(() => {
				console.log('setting', v(...args));
				set(w.value);
			});

			return () => {
				unsubs();
			};
		},
	};
}

// export function useRelay<T, Args extends unknown[]>(
// 	v: ReactiveFn<ReactivePromise<T>, Args>,
// 	...args: Args
// ): Readable<T> {
// 	return {
// 		subscribe: set => {
// 			const w = watcher(() => v(...args).value);
// 			const unsubs = w.addListener(() => {
// 				set(v(...args) as T);
// 			});

// 			return () => {
// 				unsubs();
// 			};
// 		},
// 	};
// }

// // export function useReactivePromise<T>(p: ReactivePromise<T>): Readable<T> {
// // 	return {
// // 		subscribe: set => {
// // 			const w = watcher(() => v(...args));
// // 			const unsubs = w.addListener(() => {
// // 				console.log('setting', v(...args));
// // 				set(v(...args) as T);
// // 			});

// // 			return () => {
// // 				unsubs();
// // 			};
// // 		},
// // 	};
// // }
