import { reactive } from 'signalium';

import { GroupChatClient } from '../group-chats/group-chat-client';
import { GroupChatStore } from '../group-chats/group-chat-store';
import { LogsStore } from '../p2panda/logs-store';
import { PublicKey, TopicId } from '../p2panda/types';
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
		protected logsStore: LogsStore<TopicId, Payload>,
		public client: ChatsClient,
	) {}

	async createGroup(initialMembers: PublicKey[]): Promise<GroupChatStore> {
		console.log('yaa')
		const chatId = random_hexadecimal(64);

		await this.client.createGroup(chatId);

		const groupStore = this.groupChats(chatId);

		for (const initialMember of initialMembers) {
			await groupStore.addMember(initialMember);
		}

		return groupStore;
	}

	groupChats = reactive(
		(chatId: ChatId) =>
			new GroupChatStore(this.logsStore, new GroupChatClient(), chatId),
	);

	allChatsIds = reactive(() => this.client.getGroups())

	allChatsSummaries = reactive(async ()=>{
		const chats  = await this.allChatsIds();

	})

	chatSummary = reactive((chatId: ChatId) => {
		const groupChatStore = this.groupChats(chatId);

		
	})
}
