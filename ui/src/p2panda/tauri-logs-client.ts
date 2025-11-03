import { blake2b, blake2bHex } from 'blakejs';
import Emittery, { type UnsubscribeFunction } from 'emittery';

import type { LogsClient } from './logs-client';
import type { SimplifiedHeader, SimplifiedOperation } from './simplified-types';
import type {
	Hash,
	Header,
	LogId,
	Operation,
	PublicKey,
	TopicId,
} from './types';

export class TauriLogsClient implements LogsClient {
	constructor() {}

	async myPubKey() {
		// TODO: implement with invoke
		return 'unimplemented';
	}

	async getLog(
		topicId: TopicId,
		author: PublicKey,
		logId: LogId,
	): Promise<SimplifiedOperation<any>[]> {
		// TODO: implement with invoke
		return [];
	}

	async getAuthorsForTopic(topicId: TopicId): Promise<PublicKey[]> {
		// TODO: implement with invoke
		return [];
	}

	onNewOperation(
		handler: (
			topicId: TopicId,
			author: PublicKey,
			logId: LogId,
			operation: SimplifiedOperation<any>,
		) => void,
	): UnsubscribeFunction {
		// TODO: implement with event listener
		return () => {};
	}
}
