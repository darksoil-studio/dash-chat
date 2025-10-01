import { UnsubscribeFn } from '../friends/friends-client';
import { Message, MessageContent, MessageId } from '../types';
import { UserId } from '../users/user-client';

// Client tied to a specific peer chat
export interface PeerChatClient {
	// Get the peer for this chat
	getPeer(): Promise<UserId>;
	
	/// Messages

	// Get all messages for this peer chat
	getMessages(): Promise<Message[]>;

	// Executes the handler when a new message is received
	onNewMessage(handler: (message: Message) => void): UnsubscribeFn;

	// Sends a message in this peer chat
	sendMessage(messageContent: MessageContent): Promise<MessageId>;

	/// Typing indicator

	// Sends a typing indicator signal
	sendTypingIndicatorSignal(): Promise<void>;

	// Receive typing indicator signal
	onTypingIndicatorSignal(): UnsubscribeFn;
}
