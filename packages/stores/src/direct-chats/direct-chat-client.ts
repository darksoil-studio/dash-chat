import { invoke } from '@tauri-apps/api/core';

import { AgentId, TopicId } from '../p2panda/types';
import { ChatId, MessageContent, ChatReaction } from '../types';

export interface IDirectChatClient {
	sendMessage(chatId: ChatId, content: MessageContent): Promise<void>;
}

export class DirectChatClient implements IDirectChatClient {
	chatId(peer: AgentId): Promise<ChatId> {
		return invoke('direct_chat_id', {
			peer,
		});
	}

	async sendMessage(chatId: ChatId, content: MessageContent): Promise<void> {
		return invoke('direct_chat_send_message', {
			chatId,
			content,
		});
	}

	async sendReaction(chatId: ChatId, content: ChatReaction): Promise<void> {
		return invoke('direct_chat_send_reaction', {
			chatId,
			content,
		});
	}
}
