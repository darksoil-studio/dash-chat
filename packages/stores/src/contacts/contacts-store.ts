import { ReactivePromise, reactive } from 'signalium';

import { DevicesStore } from '../devices/devices-store';
import { LogsStore } from '../p2panda/logs-store';
import { SimplifiedOperation } from '../p2panda/simplified-types';
import { ActorId, PublicKey, TopicId } from '../p2panda/types';
import { personalTopicFor } from '../topics';
import { AnnouncementPayload, Payload } from '../types';
import { IContactsClient, Profile } from './contacts-client';

export type ContactRequestId = string

export interface IncomingContactRequest {
	profile: Profile;
	actorId: ActorId;
	contactRequestId: ContactRequestId
}

export class ContactsStore {
	constructor(
		protected logsStore: LogsStore<TopicId, Payload>,
		protected devicesStore: DevicesStore,
		public client: IContactsClient,
	) {}

	myChatActorId = reactive(async () => await this.client.myChatActorId());

	myProfile = reactive(async () => {
		const myChatActorId = await this.myChatActorId();

		return await this.profiles(myChatActorId);
	});

	incomingContactRequests = reactive(async () => {
		const requests :Array<IncomingContactRequest> = [{
			actorId: await this.myChatActorId(),
			profile: (await this.myProfile())!,
			contactRequestId: '1'
		}]
		return requests
	})

	profiles = reactive(async (actorId: ActorId) => {
		const topicId = personalTopicFor(actorId);

		const operations = await this.logsStore.logsForAllAuthors(topicId);

		const log: SimplifiedOperation<Payload>[] =
			Object.values(operations)[0] || [];
			console.log(log)

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

	contactsActorIds = reactive(async () => {
		const myDeviceGroupTopic = await this.devicesStore.myDeviceGroupTopic();

		const contacts: Set<ActorId> = new Set();

		for (const [_, ops] of Object.entries(myDeviceGroupTopic)) {
			for (const op of ops) {
				if (op.body?.payload?.type === 'AddContact') {
					contacts.add(op.body.payload.payload.chat_actor_id);
				}
			}
		}

		return Array.from(contacts);
	});

	profilesForAllContacts = reactive(async () => {
		const contacts = await this.contactsActorIds();

		const profiles = await ReactivePromise.all(
			contacts.map(contact => this.profiles(contact)),
		);

		const profilesWithContacts: Array<[ActorId, Profile]> = profiles
			.filter(p => !!p)
			.map((profile, i) => [contacts[i], profile]);

		return profilesWithContacts;
	});
}
