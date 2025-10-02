import { Signal } from 'signal-polyfill';

import type { AsyncComputed } from '../signals/async-computed';
import { MemoMap } from '../signals/memo-map';
import { AsyncRelay, type AsyncResult } from '../signals/relay';
import type { LogsClient } from './logs-client';
import type { LogId, Operation, PublicKey, TopicId } from './types';

export class LogsStore {
	constructor(protected logsClient: LogsClient) {}

	myPubKey = new AsyncRelay<PublicKey>(async set => {
		const myPubKey = await this.logsClient.myPubKey();
		set(myPubKey);
	});

	authorsForTopic = new MemoMap(
		(topicId: TopicId) =>
			new AsyncRelay(async (set, get) => {
				const authors = await this.logsClient.getAuthorsForTopic(topicId);
				set(authors);

				return this.logsClient.onNewOperation(
					(operationTopicId, author, logId) => {
						if (topicId !== operationTopicId) return;
						if (authors.includes(author)) return;
						authors.push(author);
						set(authors);
					},
				);
			}),
	);

	logs = new MemoMap(
		(topicId: TopicId) =>
			new MemoMap(
				(author: PublicKey) =>
					new MemoMap(
						(logId: LogId) =>
							new AsyncRelay<Operation[]>(async (set, get) => {
								const log = await this.logsClient.getLog(
									topicId,
									author,
									logId,
								);
								set(log);

								return this.logsClient.onNewOperation(
									(
										operationTopicId,
										operationAuthor,
										operationLogId,
										operation,
									) => {
										if (topicId !== operationTopicId) return;
										if (author !== operationAuthor) return;
										if (logId !== operationLogId) return;
										log.push(operation);
										set(log);
									},
								);
							}),
					),
			),
	);

	logsForAllAuthors = new MemoMap(
		(topicId: TopicId) =>
			new MemoMap(
				(logId: LogId) =>
					new Signal.Computed<AsyncResult<Record<PublicKey, Operation[]>>>(
						() => {
							const authorsForTopic = this.authorsForTopic.get(topicId).get();
							if (authorsForTopic.status !== 'completed')
								return authorsForTopic;

							const logsForAllAuthors: Record<PublicKey, Operation[]> = {};

							for (const author in authorsForTopic) {
								const log = this.logs.get(topicId).get(author).get(logId).get();
								if (log.status !== 'completed') return log;
								logsForAllAuthors[author] = log.value;
							}
							return {
								status: 'completed',
								value: logsForAllAuthors,
							};
						},
					),
			),
	);
}

// store.logs.get(chatId).get().get("messages")
