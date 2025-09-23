import { UnsubscribeFn } from '../friends/friends-client';
import { Message, MessageContent, MessageId } from '../types';
import { UserId } from '../users/user-client';

export interface GroupInfo {
	name: string;
	description: string;
	avatar_src: string | undefined;
}

export interface GroupMember {
	userId: UserId;
	admin: boolean
}

// Client tied to a specific group chat
export interface GroupChatClient {
	/// Info

	getGroupInfo(): Promise<GroupInfo>;

	/// Members

	getMembers(): Promise<GroupMember[]>;

	addMember(userId: UserId): Promise<void>;

	removeMember(userId: UserId): Promise<void>;

	promoteToAdmin(userId: UserId): Promise<void>;

	demoteFromAdmin(userId: UserId): Promise<void>;

	/// Messages

	// Get all messages for this group chat
	getMessages(): Promise<Message[]>;

	// Executes the handler when a new message is received
	onNewMessage(handler: (message: Message) => void): UnsubscribeFn;

	// Sends a message in this group chat
	sendMessage(messageContent: MessageContent): Promise<MessageId>;

	/// Typing indicator

	// Sends a typing indicator signal
	sendTypingIndicatorSignal(): Promise<void>;

	// Receive typing indicator signal from the given user
	onTypingIndicatorSignal(handler: (userId: UserId) => void): UnsubscribeFn;
}
