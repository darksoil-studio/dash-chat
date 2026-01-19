import { reactive } from 'signalium';

import { ContactsStore } from '../contacts/contacts-store';
import { Message, MessageContent } from '../group-chats/group-chat-client';
import { LogsStore } from '../p2panda/logs-store';
import { AgentId, TopicId } from '../p2panda/types';
import { ChatId, Payload } from '../types';
import { DirectMessagesChatClient } from './direct-messages-chat-client';

// Store tied to a specific direct messages chat
export class DirectMessagesChatStore {
	constructor(
		protected logsStore: LogsStore<Payload>,
		protected contactsStore: ContactsStore,
		public client: DirectMessagesChatClient,
		public peer: AgentId,
	) {}

	chatId = reactive(async () => {
		return await this.client.chatId(this.peer);
	});

	peerProfile = reactive(async () => {
		return await this.contactsStore.myProfile();
	});

	messages = reactive(async () => {
		const messages: Array<Message> = [
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
		];

		return messages;
	});

	sendMessage(content: MessageContent) {
		return this.client.sendMessage(this.chatId, content);
	}
}
