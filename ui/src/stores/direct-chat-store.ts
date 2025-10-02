import type { AsyncSignal } from "../signals/async-computed";
import type { UnsubscribeFn } from "../signals/relay";
import type { Message, MessageContent, MessageId } from "./types";
import type { UserId } from "./users-store";

// Store tied to a specific peer chat
export interface PeerChatStore {
	// Get the peer for this chat
	peer(): AsyncSignal<UserId>;
	
	/// Messages

	// Get all messages for this peer chat
	messages(): AsyncSignal<Message[]>;

	// Sends a message in this peer chat
	sendMessage(messageContent: MessageContent): Promise<MessageId>;

	/// Typing indicator

	// Sends a typing indicator signal
	sendTypingIndicatorSignal(): Promise<void>;

	// Receive typing indicator signal
	onTypingIndicatorSignal(): UnsubscribeFn;
}
