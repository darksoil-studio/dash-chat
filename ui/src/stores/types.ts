import type { UserId } from "./users-store";


export type MessageId = string;

export type MessageContent = {
	type: 'TextMessage';
	message: string;
	replyTo: MessageId | undefined
};

export interface Message {
	id: MessageId;
	content: MessageContent;
	author: UserId;
	timestamp: number;
}
