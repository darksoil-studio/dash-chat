import {
	type ReactiveFn,
	ReactivePromise,
	reactive,
	relay,
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
		const s = (value as any)['_signal'];
		// console.log('signal', s.addListenerLazy());
		const version = (value as any)['value'];

		if (value instanceof ReactivePromise && value.value !== undefined)
			return value.value;
		// if (value instanceof Promise) {
		// 	const s = signal(value);

		// 	value.then(v => (s.value = v));

		// 	return s.value;
		// }
		return value;
	});
	return {
		subscribe: set => {
			const unsubs = w.addListener(() => {
				const s = (w.value as any)['_signal'];
				set(w.value as T);
			});

			return () => {
				unsubs();
			};
		},
	};
}

// const counter = reactive(() =>
// 	relay<number>(state => {
// 		console.log('relay');
// 		state.value = 0;

// 		const id = setInterval(() => state.value!++, 1000);

// 		return () => clearInterval(id);
// 	}),
// );

// const getInnerLoader = reactive(() => {
// 	return counter();
// });

// const getOuterLoader = reactive(async () => {
// 	const innerValue = await getInnerLoader();

// 	return innerValue + 1;
// });

// export const getText = reactive(() => {
// 	const { isPending, value } = getOuterLoader();

// 	return isPending ? 'Loading...' : value;
// });

// // const w = watcher(() => {
// // 	return getInnerLoader().value;
// // });
// // w.addListener(() => console.log('wat', w.value));

// // useSignal(counter).subscribe(console.log)
// useSignal(getOuterLoader).subscribe(console.log);
