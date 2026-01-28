import type { UnsubscribeFunction } from 'emittery';
import type { SimplifiedOperation } from './simplified-types';
import type { PublicKey } from './types';

export interface LogsClient<TOPIC_ID, PAYLOAD> {
	getAuthorsForTopic(topicId: TOPIC_ID): Promise<PublicKey[]>;

	getLog(
		topicId: TOPIC_ID,
		author: PublicKey,
	): Promise<SimplifiedOperation<PAYLOAD>[]>;

	onNewOperation(
		handler: (
			topicId: TOPIC_ID,
			operation: SimplifiedOperation<PAYLOAD>,
		) => void,
	): UnsubscribeFunction;
}


