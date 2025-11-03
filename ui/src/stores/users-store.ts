import { reactive } from 'signalium';

import type { LogsStore } from '../p2panda/logs-store';
import { type PublicKey, type TopicId } from '../p2panda/types';

export type UserId = PublicKey;

export interface User {
	publicKeys: PublicKey[];
	profile: Profile | undefined;
}

export interface Profile {
	name: string;
	avatar_src: string | undefined;
}

export interface IUsersClient {
	// Sets the profile for this user
	setProfile(profile: Profile): Promise<void>;
}

export function userTopicFor(userId: UserId): TopicId {
	return `${userId}`;
}

export class UsersStore {
	constructor(
		protected logsStore: LogsStore,
		protected usersClient: IUsersClient,
	) {}

	me = reactive(async () => {
		console.log('hay');
		const myPubKey = await this.logsStore.myPubKey();
		console.log('hay2');
		console.log(myPubKey);
		// if (myPubKey.status !== 'completed') return myPubKey;

		const user = await this.users(myPubKey);
		return user;
		// if (myUser.status !== 'completed') return myUser;
		// if (myUser.value) return myUser as AsyncResult<User>;
		// else {
		// 	return {
		// 		status: 'completed',
		// 		value: {
		// 			profile: undefined,
		// 			publicKeys: [myPubKey.value],
		// 		},
		// 	};
		// }
	});

	users = reactive(async (userId: UserId) => {
		console.log('haaa1');
		const topicId = userTopicFor(userId);
		const operations = await this.logsStore.logsForAllAuthors(
			topicId,
			'profile',
		);
		console.log('haaa3');

		// if (operations.status !== 'completed') return operations;

		const log = Object.values(operations)[0];

		const descendantSortedOperations = log.sort(
			(o1, o2) => o2.header.timestamp - o1.header.timestamp,
		);
		const lastOperation = descendantSortedOperations[0];
		console.log(log);

		if (!lastOperation || !lastOperation.body) {
			return {
				profile: undefined,
				publicKeys: [userId],
			} as User;
		}

		const profile: Profile = lastOperation.body;
		return {
			profile,
			publicKeys: [userId],
		} as User;
	});
	// new AsyncComputed<User>(() => {
	// 	const topicId = userTopicFor(userId);

	// 	const operations = this.logsStore.logsForAllAuthors
	// 		.get(topicId)
	// 		.get('profile')
	// 		.get();

	// 	if (operations.status !== 'completed') return operations;

	// 	const log = Object.values(operations.value)[0];

	// 	const descendantSortedOperations = log.sort(
	// 		(o1, o2) => o2.header.timestamp - o1.header.timestamp,
	// 	);
	// 	const lastOperation = descendantSortedOperations[0];

	// 	if (!lastOperation || !lastOperation.body) {
	// 		return {
	// 			status: 'completed',
	// 			value: {
	// 				profile: undefined,
	// 				publicKeys: [userId],
	// 			},
	// 		};
	// 	}

	// 	const profile: Profile = (lastOperation.body);
	// 	return {
	// 		status: 'completed',
	// 		value: {
	// 			profile,
	// 			publicKeys: [userId],
	// 		},
	// 	};
	// }),

	setProfile(profile: Profile): Promise<void> {
		return this.usersClient.setProfile(profile);
	}
}
