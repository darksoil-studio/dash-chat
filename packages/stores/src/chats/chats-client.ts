import { invoke } from '@tauri-apps/api/core';

import { ChatId } from '../types';

export interface IChatsClient {
	createGroupChat(chatId: ChatId): Promise<void>;
	getGroupChats(): Promise<Array<ChatId>>;
}

export class ChatsClient implements IChatsClient {
	createGroupChat(groupChatId: ChatId): Promise<void> {
		return invoke('create_group_chat', {
			groupChatId,
		});
	}

	getGroupChats(): Promise<Array<ChatId>> {
		return invoke('get_group_chats');
	}
}
