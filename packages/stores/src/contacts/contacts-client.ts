import { invoke } from '@tauri-apps/api/core';

import { type PublicKey, type TopicId } from '../p2panda/types';

export interface Profile {
	name: string;
	avatar: string | undefined;
}

export type ContactRequestId = string;

// export type MemberCode = [LongTermKeyBundle, ActorId];
export type ContactCode = string;

export interface IContactsClient {
	/// Profiles

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
