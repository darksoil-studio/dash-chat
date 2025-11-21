import { invoke } from '@tauri-apps/api/core';

import { ActorId, PublicKey, TopicId } from '../p2panda/types';
import { ChatId, Payload } from '../types';

export type ChatMessageContent = string;

export interface ChatMessage {
	content: ChatMessageContent;
	author: PublicKey;
	timestamp: number;
}

export interface IGroupChatClient {
	/// Members
	addMember(chatId: ChatId, member: PublicKey): Promise<void>;
	removeMember(chatId: ChatId, member: PublicKey): Promise<void>;

	promoteToAdministrator(chatId: ChatId, member: ActorId): Promise<void>;
	demoteFromAdministrator(chatId: ChatId, member: ActorId): Promise<void>;

	/// Messages

	sendMessage(chatId: ChatId, content: ChatMessageContent): Promise<void>;

	leaveGroup(): Promise<void>;
	deleteGroup(): Promise<void>;
}

export class GroupChatClient implements IGroupChatClient {
	addMember(chatId: ChatId, member: PublicKey): Promise<void> {
		return invoke('add_member', {
			chatId,
			member,
		});
	}
	async removeMember(chatId: ChatId, member: PublicKey): Promise<void> {}

	sendMessage(chatId: ChatId, content: ChatMessageContent): Promise<void> {
		return invoke('send_message', { chatId, content });
	}
	async promoteToAdministrator(
		chatId: ChatId,
		member: ActorId,
	): Promise<void> {}
	async demoteFromAdministrator(
		chatId: ChatId,
		member: ActorId,
	): Promise<void> {}

	async leaveGroup(): Promise<void> {
	}

	async deleteGroup(): Promise<void> {}
}

const sleep = (ms: number) =>
	new Promise(r => setTimeout(() => r(undefined), ms));
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
