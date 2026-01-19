import { ReactivePromise, reactive, relay } from 'signalium';

import { DevicesStore } from '../devices/devices-store';
import { LogsStore } from '../p2panda/logs-store';
import { SimplifiedOperation } from '../p2panda/simplified-types';
import { AgentId, PublicKey, TopicId } from '../p2panda/types';
import { personalTopicFor } from '../topics';
import { AnnouncementPayload, ContactCode, Payload } from '../types';
import { ContactRequestId, IContactsClient, Profile } from './contacts-client';

export interface ContactRequest {
	profile: Profile;
	code: ContactCode;
	// agentId: AgentId;
	// contactRequestId: ContactRequestId;
}

export class ContactsStore {
	constructor(
		protected logsStore: LogsStore<Payload>,
		protected devicesStore: DevicesStore,
		public client: IContactsClient,
	) {}

	myAgentId = reactive(async () => await this.client.myAgentId());

	myProfile = reactive(async () => {
		const myAgentId = await this.myAgentId();

		return await this.profiles(myAgentId);
	});

	private activeInboxTopics = reactive(() =>
		relay<TopicId[]>(state => {
			state.setPromise(this.client.activeInboxTopics());

			const interval = setInterval(() => {
				this.client.activeInboxTopics().then(topics => {
					if (topics.find(topic => !(state.value || []).includes(topic))) {
						state.value = topics;
					}
				});
			}, 1_000);

			return {
				deactivate() {
					clearInterval(interval);
				},
			};
		}),
	);

	contactRequests = reactive(async () => {
		const activeInboxTopics = await this.activeInboxTopics();

		const allLogs = await ReactivePromise.all(
			activeInboxTopics.map(topicId =>
				this.logsStore.logsForAllAuthors(topicId),
			),
		);
		const contacts = await this.contactsAgentIds();

		const contactRequests: ContactRequest[] = [];

		for (const log of allLogs) {
			for (const operations of Object.values(log)) {
				for (const operation of operations) {
					if (operation.body?.type !== 'Inbox') continue;
					// We have already accepted this contact request
					if (contacts.includes(operation.body.payload.payload.code.agent_id)) continue
					contactRequests.push(operation.body.payload.payload);
				}
			}
		}

		return contactRequests;
	});

	profiles = reactive(async (agentId: AgentId) => {
		const topicId = personalTopicFor(agentId);

		const operations = await this.logsStore.logsForAllAuthors(topicId);

		const log: SimplifiedOperation<Payload>[] =
			Object.values(operations)[0] || [];

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

	contactsAgentIds = reactive(async () => {
		const myDeviceGroupTopic = await this.devicesStore.myDeviceGroupTopic();

		const contacts: Set<AgentId> = new Set();

		for (const [_, ops] of Object.entries(myDeviceGroupTopic)) {
			for (const op of ops) {
				if (op.body?.payload?.type === 'AddContact') {
					contacts.add(op.body.payload.payload.agent_id);
				}
			}
		}

		return Array.from(contacts);
	});

	profilesForAllContacts = reactive(async () => {
		const contacts = await this.contactsAgentIds();

		const profiles = await ReactivePromise.all(
			contacts.map(contact => this.profiles(contact)),
		);

		const profilesWithContacts: Array<[AgentId, Profile]> = profiles
			.filter(p => !!p)
			.map((profile, i) => [contacts[i], profile]);

		return profilesWithContacts;
	});
}
