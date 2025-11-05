import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
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

	async myPubKey(): Promise<PublicKey> {
		return await invoke('my_pub_key');
	}

	async getLog(
		topicId: TopicId,
		author: PublicKey,
	): Promise<SimplifiedOperation<any>[]> {
		return invoke('get_log', { topicId, author });
	}

	async getAuthorsForTopic(topicId: TopicId): Promise<PublicKey[]> {
		return invoke('get_authors', { topicId });
	}

	onNewOperation(
		handler: (topicId: TopicId, operation: SimplifiedOperation<any>) => void,
	): UnsubscribeFunction {
		let unsubs: (() => void) | undefined;
		listen('p2panda://new-operation', e => {
			const operation = e.payload as SimplifiedOperation<any>;
			handler(operation.header.topic_id, operation);
		}).then(u => (unsubs = u));

		return () => {
			if (unsubs) unsubs();
		};
	}
}
