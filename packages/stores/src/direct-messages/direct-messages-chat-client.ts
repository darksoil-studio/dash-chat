import { ContactsStore } from '../contacts/contacts-store';
import { ChatMessageContent } from '../group-chats/group-chat-client';
import { LogsStore } from '../p2panda/logs-store';
import { TopicId } from '../p2panda/types';
import { ChatId, Payload } from '../types';

export interface IDirectMessagesChatClient {
	sendMessage(chatId: ChatId, content: ChatMessageContent): Promise<void>;
}

export class DirectMessagesChatClient implements IDirectMessagesChatClient {
	async sendMessage(
		chatId: ChatId,
		content: ChatMessageContent,
	): Promise<void> {}
}
