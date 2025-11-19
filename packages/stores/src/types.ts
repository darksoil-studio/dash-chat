import { Profile } from './contacts/contacts-client';
import {
	ActorId,
	Hash,
	LongTermKeyBundle,
	PublicKey,
	TopicId,
} from './p2panda/types';

export type ChatId = TopicId;

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

export interface InboxTopic {
	expires_at: number;
	topic: TopicId;
}

export interface MemberCode {
	actor_id: ActorId;
	key_bundle: LongTermKeyBundle;
}

export type ShareIntent = 'AddDevice' | 'AddContact';

export interface ContactCode {
	/// Pubkey and key bundle of this node: allows adding this node to encrypted spaces.
	member_code: MemberCode;
	/// Topic for receiving messages from this node during the lifetime of the QR code.
	/// The initiator will specify an InboxTopic, and the recipient will send back a QR
	/// code without an associated inbox, because after this exchange the two nodes
	/// can communicate directly.
	inbox_topic: InboxTopic | undefined;
	/// Topic for the device group of this node.
	device_space_id: TopicId;
	/// Actor ID to add to spaces
	chat_actor_id: ActorId;
	/// The intent of the QR code: whether to add this node as a contact or a device.
	share_intent: ShareIntent;
}

export type DeviceGroupPayload = {
	type: 'AddContact';
	payload: ContactCode;
};

export type Payload =
	| { type: 'Announcements'; payload: AnnouncementPayload }
	| { type: 'Chat'; payload: ChatPayload }
	| { type: 'DeviceGroupPayload'; payload: DeviceGroupPayload }
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
