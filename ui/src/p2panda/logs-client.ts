import type { UnsubscribeFunction } from 'emittery';
import type { SimplifiedOperation } from './simplified-types';
import type { LogId, PublicKey, TopicId } from './types';

export interface LogsClient {
	myPubKey(): Promise<PublicKey>;

	getAuthorsForTopic(topicId: TopicId): Promise<PublicKey[]>;

	getLog(
		topicId: TopicId,
		author: PublicKey,
	): Promise<SimplifiedOperation<any>[]>;

	onNewOperation(
		handler: (
			topicId: TopicId,
			operation: SimplifiedOperation<any>,
		) => void,
	): UnsubscribeFunction;
}


