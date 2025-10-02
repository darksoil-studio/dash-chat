import { Signal } from 'signal-polyfill';

import type { LogsStore } from '../p2panda/logs-store';
import { type PublicKey, type TopicId, decodeBody } from '../p2panda/types';
import { AsyncComputed, type AsyncSignal } from '../signals/async-computed';
import { MemoMap } from '../signals/memo-map';

export type UserId = PublicKey;

export interface User {
	publicKeys: PublicKey[];
	profile: Profile | undefined;
}

export interface Profile {
	name: string;
	avatar_src: string | undefined;
}

export interface IUsersStore {
	// My user
	me: AsyncSignal<User>;

	// Sets the profile for this user
	setProfile(profile: Profile): Promise<void>;

	// Get the user public keys and profile for the given user ID
	user(userId: UserId): AsyncSignal<User | undefined>;
}

export interface IUsersClient {
	// Sets the profile for this user
	setProfile(profile: Profile): Promise<void>;
}

export function userTopicFor(userId: UserId): TopicId {
	return `${userId}`;
}

export class UsersStore implements IUsersStore {
	constructor(
		protected logsStore: LogsStore,
		protected usersClient: IUsersClient,
	) {}

	me = new Signal.Computed(() => {
		const myPubKey = this.logsStore.myPubKey.get();
		if (myPubKey.status !== 'completed') return myPubKey;

		return this.users.get(myPubKey.value).get();
	});

	users = new MemoMap(
		(userId: UserId) =>
			new AsyncComputed<User>(() => {
				const topicId = userTopicFor(userId);

				const operations = this.logsStore.logsForAllAuthors
					.get(topicId)
					.get('profile')
					.get();

				if (operations.status !== 'completed') return operations;

				const log = Object.values(operations.value)[0];

				const descendantSortedOperations = log.sort(
					(o1, o2) => o2.header.timestamp - o1.header.timestamp,
				);
				const lastOperation = descendantSortedOperations[0];

				if (!lastOperation || !lastOperation.body) {
					return {
						status: 'completed',
						value: {
							profile: undefined,
							publicKeys: [userId],
						},
					};
				}

				const profile: Profile = decodeBody(lastOperation.body);
				return {
					status: 'completed',
					value: {
						profile,
						publicKeys: [userId],
					},
				};
			}),
	);

	user(userId: UserId) {
		return this.users.get(userId);
	}

	setProfile(profile: Profile): Promise<void> {
		return this.usersClient.setProfile(profile);
	}
}
