import { ReactivePromise, reactive, relay } from 'signalium';

import type { LogsClient } from './logs-client';
import type { SimplifiedOperation } from './simplified-types';
import type { PublicKey, TopicId } from './types';

export class LogsStore {
	constructor(protected logsClient: LogsClient) {}

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
					state.value = [...(state.value || []), operation];
				},
			);
			return () => {
				unsubs();
			};
		}),
	);

	logsForAllAuthors = reactive(async (topicId: TopicId) => {
		const authorsForTopic = await this.authorsForTopic(topicId);

		const logs = await ReactivePromise.all(
			authorsForTopic.map(author => this.logs(topicId, author)),
		);

		const logsForAllAuthors: Record<PublicKey, SimplifiedOperation<any>[]> = {};
		for (let i = 0; i < authorsForTopic.length; i++) {
			logsForAllAuthors[authorsForTopic[i]] = logs[i];
		}
		return logsForAllAuthors;
	});
}
