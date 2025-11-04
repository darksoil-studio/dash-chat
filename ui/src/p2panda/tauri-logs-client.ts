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
import { invoke } from '@tauri-apps/api/core';

export class TauriLogsClient implements LogsClient {
	constructor() { }

	async myPubKey(): Promise<PublicKey> {
		return await invoke("my_pub_key")
	}

	async getLog(
		topicId: TopicId,
		author: PublicKey,
	): Promise<SimplifiedOperation<any>[]> {
		const log: any[] = await invoke("get_log", { topicId, author });
		console.log("LOG", log);
		return []
		// return log.map(([header, body]) => ({
		// 	// TODO
		// }));
	}

	async getAuthorsForTopic(topicId: TopicId): Promise<PublicKey[]> {
		return await invoke("get_authors", { topicId })
	}

	onNewOperation(
		handler: (
			topicId: TopicId,
			operation: SimplifiedOperation<any>,
		) => void,
	): UnsubscribeFunction {
		// TODO: implement with event listener
		return () => { };
	}
}
