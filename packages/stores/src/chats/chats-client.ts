import { invoke } from '@tauri-apps/api/core';

import { ChatId } from '../types';

export interface IChatsClient {
	createGroup(chatId: ChatId): Promise<void>;
	getGroups(): Promise<Array<ChatId>>;
}

export class ChatsClient implements IChatsClient {
	createGroup(chatId: ChatId): Promise<void> {
		return invoke('create_group', {
			chatId,
		});
	}

	getGroups(): Promise<Array<ChatId>> {
		return invoke('get_groups');
	}
}
