import { ReactivePromise, reactive } from 'signalium';

import { GroupChatClient } from '../group-chats/group-chat-client';
import { GroupChatStore } from '../group-chats/group-chat-store';
import { LogsStore } from '../p2panda/logs-store';
import { PublicKey, TopicId } from '../p2panda/types';
import { ChatId, Payload } from '../types';
import { ChatsClient } from './chats-client';
import { ContactsStore } from '../contacts/contacts-store';
import { DirectMessagesChatStore } from '../direct-messages/direct-messages-chat-store';
import { DirectMessagesChatClient } from '../direct-messages/direct-messages-chat-client';

function random_hexadecimal(length: number) {
	var result = '';
	var characters = 'abcdef0123456789';
	var charactersLength = characters.length;
	for (let i = 0; i < length; i++)
		result += characters.charAt(Math.floor(Math.random() * charactersLength));
	return result;
}

export class ChatsStore {
	constructor(
		protected logsStore: LogsStore<TopicId, Payload>,
		protected contactsStore: ContactsStore,
		public client: ChatsClient,
	) {}

	async createGroup(initialMembers: PublicKey[]): Promise<GroupChatStore> {
		const chatId = random_hexadecimal(64);

		await this.client.createGroupChat(chatId);

		const groupStore = this.groupChats(chatId);

		for (const initialMember of initialMembers) {
			await groupStore.addMember(initialMember);
		}

		return groupStore;
	}

	groupChats = reactive(
		(chatId: ChatId) =>
			new GroupChatStore(this.logsStore,this.contactsStore, new GroupChatClient(), chatId),
	);

	directMessagesChats = reactive(
		(chatId: ChatId) =>
			new DirectMessagesChatStore(this.logsStore,this.contactsStore, new DirectMessagesChatClient(), chatId),
	);

	allChatsIds = reactive(() => this.client.getGroupChats());

	allChatsSummaries = reactive(async () => {
		const chatIds = await this.allChatsIds();

		const summaries = await ReactivePromise.all(
			chatIds.map(chatId => this.chatSummary(chatId)),
		);
		return summaries;
	});

	chatSummary = reactive((chatId: ChatId) => {
		const groupChatStore = this.groupChats(chatId);

		return {
			type: 'GroupChat',
			name: 'mygroup',
			chatId,
			avatar: undefined,
			lastEvent: {
				summary: 'aaa',
				timestamp: Date.now(),
			},
			unreadMessages: 1,
		} as ChatSummary;
	});
}

export interface ChatSummary {
	type: 'GroupChat' | 'DirectMessagesChat';
	chatId: TopicId;
	unreadMessages: number;
	name: string;
	avatar: string | undefined;
	lastEvent: {
		summary: string;
		timestamp: number;
	};
}
