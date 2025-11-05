import type { UnsubscribeFunction } from 'emittery';
import type { SimplifiedOperation } from './simplified-types';
import type { PublicKey, TopicId } from './types';

export interface LogsClient<TOPIC, PAYLOAD> {
	myPubKey(): Promise<PublicKey>;

	getAuthorsForTopic(topicId: TopicId): Promise<PublicKey[]>;

	getLog(
		topicId: TOPIC,
		author: PublicKey,
	): Promise<SimplifiedOperation<PAYLOAD>[]>;

	onNewOperation(
		handler: (
			topicId: TopicId,
			operation: SimplifiedOperation<PAYLOAD>,
		) => void,
	): UnsubscribeFunction;
}


