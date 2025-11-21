import { reactive } from 'signalium';

import { ContactsStore } from '../contacts/contacts-store';
import { ChatMessageContent } from '../group-chats/group-chat-client';
import { LogsStore } from '../p2panda/logs-store';
import { TopicId } from '../p2panda/types';
import { ChatId, Payload } from '../types';
import { DirectMessagesChatClient } from './direct-messages-chat-client';

// Store tied to a specific direct messages chat
export class DirectMessagesChatStore {
	constructor(
		protected logsStore: LogsStore<TopicId, Payload>,
		protected contactsStore: ContactsStore,
		public client: DirectMessagesChatClient,
		public chatId: ChatId,
	) {}

	peerProfile = reactive(async () => {
		return await this.contactsStore.myProfile();
	});

	messages = reactive(async () => {
		return [];
	});

	sendMessage(content: ChatMessageContent) {
		return this.client.sendMessage(this.chatId, content);
	}
}
