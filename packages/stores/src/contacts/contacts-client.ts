import { invoke } from '@tauri-apps/api/core';
// @ts-ignore
import { decode, encode } from 'cbor-web';

import { ActorId, type PublicKey, type TopicId } from '../p2panda/types';
import { ContactCode } from '../types';

export interface Profile {
	name: string;
	avatar: string | undefined;
}

export type ContactRequestId = string;

export function toHex(buffer: Uint8Array): string {
	return Array.prototype.map
		.call(buffer, x => ('00' + x.toString(16)).slice(-2))
		.join('');
}
export function encodeContactCode(contactCode: ContactCode): string {
	const bin = encode([
		contactCode.member_code,
		contactCode.inbox_topic,
		contactCode.device_space_id,
		contactCode.chat_actor_id,
		contactCode.share_intent,
	]);
	return toHex(bin);
}


const fromHexString = (hexString: string) =>
	Uint8Array.from(hexString.match(/.{1,2}/g)!.map(byte => parseInt(byte, 16)));
export function decodeContactCode(contactCodeString: string): ContactCode {
	const bin = fromHexString(contactCodeString);
	const [
		member_code,
		inbox_topic,
		device_space_id,
		chat_actor_id,
		share_intent,
	] = decode(bin);
	return {
		member_code,
		inbox_topic,
		device_space_id,
		chat_actor_id,
		share_intent,
	};
}

export interface IContactsClient {
	/// Profiles

	myChatActorId(): Promise<ActorId>;

	// Sets the profile for this user
	setProfile(profile: Profile): Promise<void>;

	/// contacts

	// Creates a new contact code to be shared
	createContactCode(): Promise<ContactCode>;

	// getContacts(): Promise<Array<PublicKey>>;

	// Remove contact
	addContact(code: ContactCode): Promise<void>;

	// Remove contact
	// removeContact(contact: ContactId): Promise<void>;

	/// Contact Requests

	// // Send contact request to the given user
	// sendContactRequest(userId: UserId): Promise<void>;

	// // Accept contact request for the given user
	// acceptContactRequest(userId: UserId): Promise<void>;

	// // Reject contact request for the given user
	// rejectContactRequest(userId: UserId): Promise<void>;

	// // Cancel contact request for the given user
	// cancelContactRequest(userId: UserId): Promise<void>;
}

export class ContactsClient implements IContactsClient {
	myChatActorId(): Promise<ActorId> {
		return invoke('my_chat_actor_id');
	}

	async setProfile(profile: Profile): Promise<void> {
		return invoke('set_profile', {
			profile,
		});
	}

	createContactCode(): Promise<ContactCode> {
		return invoke('create_contact_code');
	}

	addContact(contactCode: ContactCode): Promise<void> {
		return invoke('add_contact', {
			contactCode,
		});
	}

	// getContacts(): Promise<Array<PublicKey>> {
	// 	return invoke('get_contacts');
	// }

	// removeContact(contactId: ContactId): Promise<void> {
	// 	return invoke('remove_contact', {
	// 		contactId,
	// 	});
	// }
}
