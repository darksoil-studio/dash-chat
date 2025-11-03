import { reactive, relay } from 'signalium';

import type { LogsClient } from './logs-client';
import type { SimplifiedOperation } from './simplified-types';
import type { LogId, PublicKey, TopicId } from './types';

export class LogsStore {
	constructor(protected logsClient: LogsClient) {}

	myPubKey = reactive(() => this.logsClient.myPubKey());

	authorsForTopic = reactive((topicId: TopicId) =>
		relay<PublicKey[]>(state => {
			console.log('haii');
			const fetchAuthors = async () => {
				const authors = await this.logsClient.getAuthorsForTopic(topicId);
				state.value = authors;
			};
			fetchAuthors();

			const unsubs = this.logsClient.onNewOperation(
				(operationTopicId, author, _logId) => {
					console.log('newop', operationTopicId);
					if (topicId !== operationTopicId) return;
					const authors = state.value || [];
					if (authors.includes(author)) return;
					authors.push(author);
					state.value = authors;
				},
			);

			return unsubs;
		}),
	);

	logs = reactive((topicId: TopicId, author: PublicKey, logId: LogId) =>
		relay<SimplifiedOperation<any>[]>(state => {
			const fetchLog = async () => {
				const log = await this.logsClient.getLog(topicId, author, logId);
				state.value = log;
			};
			fetchLog();
			return {
				deactivate: () =>
					this.logsClient.onNewOperation(
						(operationTopicId, operationAuthor, operationLogId, operation) => {
							if (topicId !== operationTopicId) return;
							if (author !== operationAuthor) return;
							if (logId !== operationLogId) return;
							const log = state.value || [];
							log.push(operation);
							state.value = log;
						},
					),
			};
		}),
	);

	logsForAllAuthors = reactive(async (topicId: TopicId, logId: LogId) => {
		console.log('b');
		const authorsForTopic = await this.authorsForTopic(topicId);
		console.log('b2');

		const logsForAllAuthors: Record<PublicKey, SimplifiedOperation<any>[]> = {};

		for (const author of authorsForTopic) {
			const log = await this.logs(topicId, author, logId);
			// if (log.status !== 'completed') return log;
			logsForAllAuthors[author] = log;
		}
		return logsForAllAuthors;
	});
}

// store.logs.get(chatId).get().get("messages")
