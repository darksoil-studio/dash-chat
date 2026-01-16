import { Profile } from './contacts/contacts-client';
import {
	AgentId,
	DeviceId,
	Hash,
	LongTermKeyBundle,
	PublicKey,
	TopicId,
} from './p2panda/types';

export type ChatId = TopicId;

export interface SpaceControlMessage {
	hash: Hash;
	author: AgentId;
	timestamp: number;
	// spaces_args: SpacesArgs,
}

export type AnnouncementPayload = { type: 'SetProfile'; payload: Profile };
export type ChatPayload = Array<SpaceControlMessage>;

export interface InboxTopic {
	expires_at: number;
	topic: TopicId;
}

export type ShareIntent = 'AddDevice' | 'AddContact';

export interface ContactCode {
	/// Pubkey of this node: allows adding this node to groups.
	device_pubkey: DeviceId;
	/// Agent ID to add to spaces
	agent_id: AgentId;
	inbox_topic: InboxTopic | undefined;
	/// The intent of the QR code: whether to add this node as a contact or a device.
	share_intent: ShareIntent;
}

export type DeviceGroupPayload = {
	type: 'AddContact';
	payload: ContactCode;
};

export type InboxPayload = {
	type: 'Contact';
	payload: ContactCode;
};

export type Payload =
	| { type: 'Announcements'; payload: AnnouncementPayload }
	| { type: 'Chat'; payload: ChatPayload }
	| { type: 'DeviceGroupPayload'; payload: DeviceGroupPayload }
	| { type: 'Inbox' ; payload: InboxPayload };

export type MessageId = string;

// export type MessageContent = {
// 	type: 'TextMessage';
// 	message: string;
// 	replyTo: MessageId | undefined;
// };

// export interface Message {
// 	id: MessageId;
// 	content: MessageContent;
// 	author: PublicKey;
// 	timestamp: number;
// }
