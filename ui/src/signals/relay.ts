import { Signal } from 'signal-polyfill';
import { AsyncComputed } from 'signal-utils/async-computed';
import { effect } from 'signal-utils/subtle/microtask-effect';

// import { load } from './async-computed';

// export function asyncRelay<T>(
// 	maker: AsyncRelayMakerFn<T>,
// ): Promise<Signal.Computed<T> & { subscribe: (v: T) => void }> {
// 	return new Promise(async (resolve, reject) => {
// 		let unsubs: UnsubscribeFn | void;
// 		const signal = new Signal.State<T | undefined>(undefined, {
// 			[Signal.subtle.unwatched]: () => {
// 				if (unsubs) unsubs();
// 			},
// 			// [Signal.subtle.unwatched]: () => {
// 			// 	if (unsubs) unsubs();
// 			// },
// 		});
// 		const set = (value: T) => signal.set(value);
// 		unsubs = await maker(set, () => signal.get()!);
// 	});
// }

export type AsyncResult<T, E = unknown> =
	| {
			status: 'initial';
	  }
	| {
			status: 'loading';
	  }
	| {
			status: 'completed';
			value: T;
	  }
	| {
			status: 'error';
			error: E;
	  };

export type UnsubscribeFn = () => void;

export type AsyncRelayMakerFn<T> = (
	set: (value: T) => void,
	get: () => AsyncResult<T>,
) => Promise<UnsubscribeFn | void>;

export class AsyncRelay<T, E = unknown> extends AsyncComputed<T> {
	#signal: Signal.State<AsyncResult<T, E>> = new Signal.State({
		status: 'initial',
	});
	#unsubscribeFn: UnsubscribeFn | void = undefined;
	constructor(protected maker: AsyncRelayMakerFn<T>) {
		super(() => {
			const value = this.#signal.get();
			switch (value.status) {
				case 'initial':
				case 'loading':
					return new Promise(r => {});
				case 'completed':
					return Promise.resolve(value.value);
				case 'error':
					return Promise.reject(value.error);
			}
		});
		this.#signal = new Signal.State(
			{
				status: 'initial',
			},
			{
				[Signal.subtle.watched]: async () => {
					this.#signal.set({
						status: 'loading',
					});
					try {
						this.#unsubscribeFn = await this.maker(
							value =>
								this.#signal.set({
									status: 'completed',
									value,
								}),
							() => this.#signal.get(),
						);
					} catch (error: unknown) {
						this.#signal.set({
							status: 'error',
							error: error as E,
						});
					}
				},
				[Signal.subtle.unwatched]: () => {
					if (this.#unsubscribeFn) {
						this.#unsubscribeFn();
					}
				},
			},
		);
	}

	// get(): AsyncResult<T, E> {
	// 	return this.#signal.get();
	// }

	// subscribe(fn: (value: Promise<T>) => void): UnsubscribeFn {
	// 	return effect(() => {
	// 		const value = this.get();
	// 		switch (value.status) {
	// 			case 'initial':
	// 			case 'loading':
	// 				fn(new Promise(r => {}));
	// 				break;
	// 			case 'error':
	// 				fn(Promise.reject(value.error));
	// 				break;
	// 			case 'completed':
	// 				fn(Promise.resolve(value.value));
	// 				break;
	// 		}
	// 	});
	// }
}
