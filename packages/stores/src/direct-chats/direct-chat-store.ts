import { reactive } from 'signalium';

import { ContactsStore } from '../contacts/contacts-store';
import { Message } from '../group-chats/group-chat-client';
import { LogsStore } from '../p2panda/logs-store';
import { AgentId } from '../p2panda/types';
import { MessageContent, Payload } from '../types';
import { toPromise } from '../utils/to-promise';
import { DirectChatClient } from './direct-chat-client';

// Store tied to a specific direct chat
export class DirectChatStore {
	constructor(
		protected logsStore: LogsStore<Payload>,
		protected contactsStore: ContactsStore,
		public client: DirectChatClient,
		public peer: AgentId,
	) {}

	chatId = reactive(async () => {
		return await this.client.chatId(this.peer);
	});

	peerProfile = reactive(async () => {
		const request = await this.getContactRequest();
		if (request) return request.profile;
		return await this.contactsStore.profiles(this.peer);
	});

	getContactRequest = reactive(async () => {
		const contactRequests = await this.contactsStore.contactRequests();
		return contactRequests.find(cr => cr.code.agent_id === this.peer);
	});

	messages = reactive(async () => {
		const chatId = await this.chatId();
		const logs = await this.logsStore.logsForAllAuthors(chatId);

		const messages: Array<Message> = [];

		for (const [author, operations] of Object.entries(logs)) {
			for (const operation of operations) {
				const body = operation.body;
				if (body?.type === 'Chat' && body.payload.type === 'Message') {
					messages.push({
						content: body.payload.payload,
						author,
						timestamp: operation.header.timestamp,
					});
				}
			}
		}

		// Sort messages by timestamp (ascending order)
		messages.sort((a, b) => a.timestamp - b.timestamp);

		return messages;
	});

	async sendMessage(content: MessageContent) {
		const chatId = await toPromise(this.chatId);
		const myDeviceId = await toPromise(this.contactsStore.myDeviceId);
		const promise = new Promise(resolve => {
			this.logsStore.logsClient.onNewOperation((topicId, op) => {
				if (topicId !== chatId) return;
				if (op.body?.payload.type !== 'Message') return;
				if (op.header.public_key !== myDeviceId) return;
				if (op.body.payload.payload !== content) return;

				resolve(undefined);
			});
		});
		await this.client.sendMessage(chatId, content);
		return promise 
	}
}
