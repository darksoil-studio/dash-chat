import { invoke } from '@tauri-apps/api/core';

import { LogsStore } from '../p2panda/logs-store';
import { PublicKey, TopicId } from '../p2panda/types';
import { ChatId, Payload } from '../types';

export interface GroupInfo {
	name: string;
	description: string;
	avatar_src: string | undefined;
}

export interface GroupMember {
	publicKeys: Array<PublicKey>;
	admin: boolean;
}

export type ChatMessageContent = string;

export interface ChatMessage {
	content: ChatMessageContent;
	author: PublicKey;
	timestamp: number;
}

export interface IGroupChatClient {
	addMember(chatId: ChatId, member: PublicKey): Promise<void>;
	sendMessage(chatId: ChatId, content: ChatMessageContent): Promise<void>;
}

export class GroupChatClient implements IGroupChatClient {
	addMember(chatId: ChatId, member: PublicKey): Promise<void> {
		return invoke('add_member', {
			chatId,
			member,
		});
	}

	sendMessage(chatId: ChatId, content: ChatMessageContent): Promise<void> {
		return invoke('send_message', { chatId, content });
	}
}

// // Store tied to a specific group chat
// export interface GroupChatStore {
// 	/// Info

// 	groupInfo(): AsyncSignal<GroupInfo>;

// 	/// Members

// 	members(): AsyncSignal<GroupMember[]>;

// 	addMember(userId: UserId): Promise<void>;

// 	removeMember(userId: UserId): Promise<void>;

// 	promoteToAdmin(userId: UserId): Promise<void>;

// 	demoteFromAdmin(userId: UserId): Promise<void>;

// 	/// Messages

// 	// Get all messages for this group chat
// 	messages(): AsyncSignal<Message[]>;

// 	// Sends a message in this group chat
// 	sendMessage(messageContent: MessageContent): Promise<MessageId>;

// 	/// Typing indicator

// 	// Sends a typing indicator signal
// 	sendTypingIndicatorSignal(): Promise<void>;

// 	// Receive typing indicator signal from the given user
// 	onTypingIndicatorSignal(handler: (userId: UserId) => void): UnsubscribeFn;
// }
