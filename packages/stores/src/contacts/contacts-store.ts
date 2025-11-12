import { ReactivePromise, reactive } from 'signalium';

import { LogsStore } from '../p2panda/logs-store';
import { SimplifiedOperation } from '../p2panda/simplified-types';
import { PublicKey, TopicId } from '../p2panda/types';
import { AnnouncementPayload, Payload } from '../types';
import { IContactsClient, Profile } from './contacts-client';

export function personalTopicFor(publicKey: PublicKey): TopicId {
	return `${publicKey}`;
}

export class ContactsStore {
	constructor(
		protected logsStore: LogsStore<TopicId, Payload>,
		public client: IContactsClient,
	) {}

	myPubKey = reactive(async () => {
		const pk = await this.logsStore.myPubKey();
		return pk;
	});

	myProfile = reactive(async () => {
		const myPubKey = await this.logsStore.myPubKey();

		return await this.profiles(myPubKey);
	});

	myMemberCode = reactive(async () => this.client.myMemberCode());

	profiles = reactive(async (publicKey: PublicKey) => {
		const topicId = personalTopicFor(publicKey);
		const operations = await this.logsStore.logsForAllAuthors(topicId);

		const log: SimplifiedOperation<Payload>[] = operations[publicKey] || [];

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
			return undefined;
		}

		const profile: Profile = lastOperation[1];
		return profile;
	});

	contacts = reactive(async () => this.client.getContacts());

	profilesForAllContacts = reactive(async () => {
		const contacts = await this.contacts();

		const profiles = await ReactivePromise.all(
			contacts.map(contact => this.profiles(contact)),
		);

		const profilesWithContacts: Array<[PublicKey, Profile]> = profiles
			.filter(p => !!p)
			.map((profile, i) => [contacts[i], profile]);

		return profilesWithContacts;
	});
}
