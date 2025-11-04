import { reactive, ReactivePromise, relay } from 'signalium';

import type { LogsClient } from './logs-client';
import type { SimplifiedOperation } from './simplified-types';
import type { LogId, PublicKey, TopicId } from './types';

export class LogsStore {
	constructor(protected logsClient: LogsClient) { }

	myPubKey = reactive(() => this.logsClient.myPubKey());

	authorsForTopic = reactive((topicId: TopicId) =>
		relay<PublicKey[]>(state => {
			const fetchAuthors = async () => {
				const authors = await this.logsClient.getAuthorsForTopic(topicId);
				state.value = authors;
			};
			fetchAuthors();

			const unsubs = this.logsClient.onNewOperation(
				(operationTopicId, operation) => {
					if (topicId !== operationTopicId) return;
					const authors = state.value || [];
					const author = operation.header.public_key;
					if (authors.includes(author)) return;
					state.value = [...(state.value || []), author];
				},
			);

			return unsubs;
		}),
	);

	logs = reactive((topicId: TopicId, author: PublicKey) =>
		relay<SimplifiedOperation<any>[]>(state => {
			const fetchLog = async () => {
				const log = await this.logsClient.getLog(topicId, author);
				state.value = log;

			};
			fetchLog();

			const unsubs = this.logsClient.onNewOperation(
				(operationTopicId, operation) => {
					if (topicId !== operationTopicId) return;
					state.value = [...(state.value || []), operation]
				},
			);
			return () => {
				unsubs()
			};
		}),
	);

	logsForAllAuthors = reactive((topicId: TopicId) => {
		const authorsForTopic = this.authorsForTopic(topicId);
		if (!authorsForTopic.isReady) return authorsForTopic;

		const logsForAllAuthors: Record<PublicKey, SimplifiedOperation<any>[]> = {};
		for (const author of authorsForTopic.value) {
			const log = this.logs(topicId, author);
			if (!log.isReady) return log;
			logsForAllAuthors[author] = log.value;
		}
		return ReactivePromise.resolve(logsForAllAuthors);
	});
}
