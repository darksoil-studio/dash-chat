import { Hash } from './p2panda/types';
import type { Profile, UserId } from './users-client';

export type ChatId = Hash;

export type AnnouncementPayload = { type: 'SetProfile'; payload: Profile };

export type InboxPayload =
	| { type: 'JoinGroup'; payload: ChatId }
	| { type: 'Friend' };

export type Payload =
	| { type: 'Announcements'; payload: AnnouncementPayload }
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
	author: UserId;
	timestamp: number;
}
