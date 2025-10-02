import Emittery from 'emittery';

import type { UnsubscribeFn } from '../signals/relay';
import type { LogId, Operation, PublicKey, TopicId } from './types';

export interface LogsClient {
	myPubKey(): Promise<PublicKey>;

	getAuthorsForTopic(topicId: TopicId): Promise<PublicKey[]>;

	getLog(
		topicId: TopicId,
		author: PublicKey,
		logId: LogId,
	): Promise<Operation[]>;

	onNewOperation(
		handler: (topicId: TopicId, author: PublicKey, logId: LogId, operation: Operation) => void,
	): UnsubscribeFn;
}

