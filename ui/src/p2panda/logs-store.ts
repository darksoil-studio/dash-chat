import { AsyncComputed } from '../signals/async-computed';
import { MemoMap } from '../signals/memo-map';
import { AsyncRelay, type AsyncResult } from '../signals/relay';
import type { LogsClient } from './logs-client';
import type { SimplifiedOperation } from './simplified-types';
import type { LogId, Operation, PublicKey, TopicId } from './types';

export class LogsStore {
	constructor(protected logsClient: LogsClient) { }

	myPubKey = new AsyncComputed<PublicKey>(async () =>
		this.logsClient.myPubKey(),
	);

	authorsForTopic = new MemoMap(
		(topicId: TopicId) =>
			new AsyncRelay<PublicKey[]>(async set => {
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
							new AsyncRelay<SimplifiedOperation<any>[]>(async (set, get) => {
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
					new AsyncComputed<Record<PublicKey, SimplifiedOperation<any>[]>>(
						async () => {
							const authorsForTopic =
								await this.authorsForTopic.get(topicId).complete;
							// if (authorsForTopic.status !== 'completed')
							// 	return authorsForTopic;

							const logsForAllAuthors: Record<
								PublicKey,
								SimplifiedOperation<any>[]
							> = {};

							for (const author of authorsForTopic) {
								const log = await this.logs.get(topicId).get(author).get(logId)
									.complete;
								// if (log.status !== 'completed') return log;
								logsForAllAuthors[author] = log;
							}
							return logsForAllAuthors;
						},
					),
			),
	);
}

// store.logs.get(chatId).get().get("messages")
