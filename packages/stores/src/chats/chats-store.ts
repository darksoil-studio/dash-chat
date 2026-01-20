import { ReactivePromise, reactive } from 'signalium';

import { ContactsStore } from '../contacts/contacts-store';
import { DirectMessagesChatClient } from '../direct-messages/direct-messages-chat-client';
import { DirectMessagesChatStore } from '../direct-messages/direct-messages-chat-store';
import { GroupChatClient } from '../group-chats/group-chat-client';
import { GroupChatStore } from '../group-chats/group-chat-store';
import { LogsStore } from '../p2panda/logs-store';
import { AgentId, PublicKey, TopicId } from '../p2panda/types';
import { ChatId, Payload } from '../types';
import { ChatsClient } from './chats-client';

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
		protected logsStore: LogsStore<Payload>,
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
			new GroupChatStore(
				this.logsStore,
				this.contactsStore,
				new GroupChatClient(),
				chatId,
			),
	);

	directMessagesChats = reactive(
		(peer: AgentId) =>
			new DirectMessagesChatStore(
				this.logsStore,
				this.contactsStore,
				new DirectMessagesChatClient(),
				peer,
			),
	);

	allChatsIds = reactive(async () => {
		const contacts = await this.contactsStore.contactsAgentIds();
		// Combine and deduplicate
		return contacts;
	});

	allChatsSummaries = reactive(async () => {
		const chatIds = await this.allChatsIds();

		let summaries = await ReactivePromise.all(
			chatIds.map(chatId => this.chatSummary(chatId)),
		);

		const pendingRequests = await this.contactsStore.contactRequests();

		const pendingRequestsSummaries: ChatSummary[] = pendingRequests.map(
			pendingRequest => ({
				type: 'ContactRequest',
				chatId: pendingRequest.code.agent_id,
				name: pendingRequest.profile.name,
				avatar: pendingRequest.profile.avatar,
				lastEvent: {
					summary: '',
					timestamp: Date.now(),
				},
				unreadMessages: 1,
			}),
		);

		summaries = [...summaries, ...pendingRequestsSummaries];
		summaries.sort((a, b) => b.lastEvent.timestamp - a.lastEvent.timestamp);

		return summaries;
	});

	chatSummary = reactive(async (chatId: ChatId) => {
		const profile = await this.contactsStore.profiles(chatId);

		return {
			type: 'DirectMessagesChat',
			chatId,
			name: profile?.name,
			avatar: profile?.avatar,
			lastEvent: {
				summary: '',
				timestamp: Date.now(),
			},
			unreadMessages: 0,
		} as ChatSummary;
	});
}

export interface ChatSummary {
	type: 'GroupChat' | 'DirectMessagesChat' | 'ContactRequest';
	chatId: TopicId;
	unreadMessages: number;
	name: string;
	avatar: string | undefined;
	lastEvent: {
		summary: string;
		timestamp: number;
	};
}
