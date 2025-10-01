import { Signal } from 'signal-polyfill';
import { effect } from 'signal-utils/subtle/microtask-effect';

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
) => Promise<UnsubscribeFn | undefined>;

export class AsyncRelay<T, E = unknown> {
	#signal: Signal.State<AsyncResult<T, E>>;
	#unsubscribeFn: UnsubscribeFn | undefined;
	constructor(protected maker: AsyncRelayMakerFn<T>) {
		this.#signal = new Signal.State(
			{
				status: 'initial',
			},
			{
				[Signal.subtle.watched]: async () => {
					console.log('watched');
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

	get(): AsyncResult<T, E> {
		return this.#signal.get();
	}

	subscribe(fn: (value: AsyncResult<T, E>) => void) {
		return effect(() => {
			const value = this.get();
			fn(value);
		});
	}
}
