import { blake2b, blake2bHex } from 'blakejs';
import { encode } from 'cbor';
import Emittery from 'emittery';

import type { LogsClient } from '../../p2panda/logs-client';
import type {
	Hash,
	Header,
	LogId,
	Operation,
	PublicKey,
	TopicId,
} from '../../p2panda/types';
import type { UnsubscribeFn } from '../../signals/relay';

export function hash<T>(obj: T): Hash {
	return blake2bHex(encode(obj));
}

export class IndexedDBLogsClient implements LogsClient {
	emitter = new Emittery();

	constructor(protected _myPubKey: PublicKey) {}

	async myPubKey() {
		return this._myPubKey;
	}

	getItems() {
		const items = { ...localStorage };
		return items;
	}

	async getLog(
		topicId: TopicId,
		author: PublicKey,
		logId: LogId,
	): Promise<Operation[]> {
		const logKey = `${topicId}/${author}/${logId}`;
		const items = this.getItems();

		const operations = Object.entries(items).filter(([key, value]) =>
			key.startsWith(logKey),
		);

		return operations.map(([k, v]) => JSON.parse(v));
	}

	async getAuthorsForTopic(topicId: TopicId): Promise<PublicKey[]> {
		const logKey = `${topicId}/authors`;
		const items = this.getItems();

		const operations = Object.entries(items).filter(([key, value]) =>
			key.startsWith(logKey),
		);

		return operations.map(([k, v]) => v);
	}

	async create<T>(topicId: TopicId, logId: LogId, body: T) {
		const log = await this.getLog(topicId, this._myPubKey, logId);
		const descendantOperations = log.sort(
			(o1, o2) => o2.header.timestamp - o1.header.timestamp,
		);
		const lastOperation = descendantOperations[0];

		const encodedBody = encode(body);
		const payload_hash = hash(body);

		const header: Header = {
			backlink: lastOperation?.hash,
			payload_hash,
			payload_size: encodedBody.length,
			previous: [],
			public_key: this._myPubKey,
			seq_num: lastOperation ? lastOperation.header.seq_num + 1 : 0,
			signature: undefined,
			timestamp: Date.now() * 1000,
			version: 0,
		};

		const headerHash = hash(header);

		const operation: Operation = {
			body: encodedBody,
			hash: headerHash,
			header,
		};

		const logKey = `${topicId}/${this._myPubKey}/${logId}/${header.seq_num}`;
		localStorage.setItem(logKey, JSON.stringify(operation));

		const authorsLogKey = `${topicId}/authors/${this._myPubKey}`;
		localStorage.setItem(authorsLogKey, this._myPubKey);

		this.emitter.emit('new-operation', {
			topicId,
			author: this._myPubKey,
			logId,
			operation,
		});
	}

	onNewOperation(
		handler: (
			topicId: TopicId,
			author: PublicKey,
			logId: LogId,
			operation: Operation,
		) => void,
	): UnsubscribeFn {
		return this.emitter.on('new-operation', event => {
			handler(event.topicId, event.author, event.logId, event.operation);
		});
	}
}
