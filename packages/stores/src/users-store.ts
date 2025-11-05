import { ReactivePromise, reactive } from 'signalium';

import type { LogsStore } from './p2panda/logs-store';
import type { SimplifiedOperation } from './p2panda/simplified-types';
import { type TopicId } from './p2panda/types';
import { AnnouncementPayload, Payload } from './types';
import type { IUsersClient, Profile, User, UserId } from './users-client';

export function userTopicFor(userId: UserId): TopicId {
	return userId;
}

export class UsersStore {
	constructor(
		protected logsStore: LogsStore,
		public client: IUsersClient,
	) {}

	me = reactive(async () => {
		const myPubKey = await this.logsStore.myPubKey();

		return this.users(myPubKey);
	});

	users = reactive(async (userId: UserId) => {
		const topicId = userTopicFor(userId);
		const operations = await this.logsStore.logsForAllAuthors(topicId);

		const log: SimplifiedOperation<Payload>[] = Object.values(operations)[0];

		const setProfiles: Array<[number, Profile]> = log
			.filter(
				l =>
					l.body?.type === 'Announcements' &&
					l.body.payload.type === 'SetProfile',
			)
			.map(l => [
				l.header.timestamp,
				((l.body! as any).payload as AnnouncementPayload).payload,
			]);

		const descendantSortedOperations = setProfiles.sort(
			(o1, o2) => o2[0] - o1[0],
		);
		const lastOperation = descendantSortedOperations[0];

		if (!lastOperation) {
			return {
				profile: undefined,
				publicKeys: [userId],
			} as User;
		}

		const profile: Profile = lastOperation[1];
		return {
			profile,
			publicKeys: [userId],
		} as User;
	});
}
