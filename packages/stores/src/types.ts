import { Profile } from './contacts/contacts-client';
import { ActorId, Hash, PublicKey } from './p2panda/types';

export type ChatId = Hash;

export interface SpaceControlMessage {
	hash: Hash;
	author: ActorId;
	timestamp: number;
	// spaces_args: SpacesArgs,
}

export type AnnouncementPayload = { type: 'SetProfile'; payload: Profile };
export type ChatPayload = Array<SpaceControlMessage>;

export type InboxPayload =
	| { type: 'JoinGroup'; payload: ChatId }
	| { type: 'Contact' };

export type Payload =
	| { type: 'Announcements'; payload: AnnouncementPayload }
	| { type: 'Chat'; payload: ChatPayload }
	| { type: 'Inbox' };

export type MessageId = string;

export type MessageContent = {
	type: 'TextMessage';
	message: string;
	replyTo: MessageId | undefined;
};

export interface Message {
	id: MessageId;
	content: MessageContent;
	author: PublicKey;
	timestamp: number;
}
