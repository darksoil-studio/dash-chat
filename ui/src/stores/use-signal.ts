import { type ReactiveFn, ReactivePromise, watcher } from 'signalium';
import { type Readable } from 'svelte/store';

export function useSignal<T, Args extends unknown[]>(
	v: ReactiveFn<T, Args>,
	...args: Args
): Readable<T> {
	const w = watcher(() => {
		const value = v(...args);
		const s = (value as any)['_signal'];
		const version = (value as any)['value'];

		if (value instanceof ReactivePromise && value.value !== undefined)
			return value.value;
		return value;
	});
	return {
		subscribe: set => {
			const unsubs = w.addListener(() => {
				if (typeof w.value === 'object') {
					set({ ...w.value } as T);
				} else {
					set(w.value as T);
				}
			});

			return () => {
				unsubs();
			};
		},
	};
}
