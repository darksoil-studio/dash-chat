import { UserId } from "../users-client";

export interface GroupInfo {
	name: string;
	description: string;
	avatar_src: string | undefined;
}

export interface GroupMember {
	userId: UserId;
	admin: boolean
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
