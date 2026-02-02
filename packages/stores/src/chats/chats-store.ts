import { ReactivePromise, reactive } from 'signalium';

import { fullName } from '../contacts/contacts-client';
import { ContactsStore } from '../contacts/contacts-store';
import { DirectChatClient } from '../direct-chats/direct-chat-client';
import { DirectChatStore } from '../direct-chats/direct-chat-store';
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

	directChats = reactive(
		(peer: AgentId) =>
			new DirectChatStore(
				this.logsStore,
				this.contactsStore,
				new DirectChatClient(),
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

		let summaries = await Promise.all(
			chatIds.map(chatId => this.chatSummary(chatId)),
		);

		const pendingRequests = await this.contactsStore.contactRequests();

		// Deduplicate by agent_id
		const uniquePendingRequests = pendingRequests.filter(
			(request, index, self) =>
				self.findIndex(r => r.code.agent_id === request.code.agent_id) ===
				index,
		);

		const pendingRequestsSummaries: ChatSummary[] = uniquePendingRequests.map(
			pendingRequest => ({
				type: 'ContactRequest',
				chatId: pendingRequest.code.agent_id,
				name: fullName(pendingRequest.profile),
				avatar: pendingRequest.profile.avatar,
				lastEvent: {
					summary: '',
					timestamp: pendingRequest.timestamp,
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
		const directChat = this.directChats(chatId);
		const message = await directChat.lastMessage();
		const unreadCount = await directChat.unreadCount();

		const lastEvent = message
			? {
					summary: message.content,
					timestamp: message.timestamp,
				}
			: {
					summary: 'contact_added',
					timestamp: await this.contactsStore.contactAddedTimestamp(chatId),
				};

		return {
			type: 'DirectChat',
			chatId,
			name: fullName(profile!),
			avatar: profile?.avatar,
			lastEvent,
			unreadMessages: unreadCount,
		} as ChatSummary;
	});
}

export interface ChatSummary {
	type: 'GroupChat' | 'DirectChat' | 'ContactRequest';
	chatId: TopicId;
	unreadMessages: number;
	name: string;
	avatar: string | undefined;
	lastEvent: {
		summary: string;
		timestamp: number;
	};
}
