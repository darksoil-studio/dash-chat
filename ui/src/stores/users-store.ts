import { ReactivePromise, reactive } from 'signalium';

import type { LogsStore } from '../p2panda/logs-store';
import { type TopicId } from '../p2panda/types';
import type { IUsersClient, Profile, User, UserId } from './users-client';
import type { SimplifiedOperation } from '../p2panda/simplified-types';

export function userTopicFor(userId: UserId): TopicId {
	return `${userId}`;
}

export class UsersStore {
	constructor(
		protected logsStore: LogsStore,
		protected usersClient: IUsersClient,
	) {}

	me = reactive(() => {
		const myPubKey = this.logsStore.myPubKey();
		if (!myPubKey.isReady) return myPubKey as any as ReactivePromise<User | void>;

		return this.users(myPubKey.value);
	});

	users = reactive((userId: UserId) => {
		const topicId = userTopicFor(userId);
		const operations = this.logsStore.logsForAllAuthors(topicId, userId);
		if (!operations.isReady) return operations as any as ReactivePromise<User | void>;

		const log: SimplifiedOperation<any>[] = Object.values(operations.value)[0];

		const descendantSortedOperations = log.sort(
			(o1, o2) => o2.header.timestamp - o1.header.timestamp,
		);
		const lastOperation = descendantSortedOperations[0];

		if (!lastOperation || !lastOperation.body) {
			return ReactivePromise.resolve({
				profile: undefined,
				publicKeys: [userId],
			} as User);
		}

		const profile: Profile = lastOperation.body;
		return ReactivePromise.resolve({
			profile,
			publicKeys: [userId],
		} as User);
	});
}
