import { invoke } from '@tauri-apps/api/core';

import { AgentId, TopicId } from '../p2panda/types';
import { ChatId, MessageContent, Payload } from '../types';

export interface IDirectMessagesChatClient {
	sendMessage(chatId: ChatId, content: MessageContent): Promise<void>;
}

export class DirectMessagesChatClient implements IDirectMessagesChatClient {
	chatId(peer: AgentId): Promise<ChatId> {
		return invoke('direct_message_chat_id', {
			peer,
		});
	}

	async sendMessage(chatId: ChatId, content: MessageContent): Promise<void> {
		return invoke('direct_messages_send_message', {
			chatId,
			content,
		});
	}
}
