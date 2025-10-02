import { Signal } from 'signal-polyfill';

import type { AsyncRelay, AsyncResult } from './relay';

export class AsyncComputed<T> extends Signal.Computed<AsyncResult<T>> {}
export type AsyncSignal<T> =
	| Signal.Computed<AsyncResult<T>>
	| Signal.State<AsyncResult<T>>
	| AsyncRelay<T>;
