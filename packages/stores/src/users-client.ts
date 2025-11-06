import { invoke } from '@tauri-apps/api/core';

import { type PublicKey, type TopicId } from './p2panda/types';

export type UserId = PublicKey;

export interface User {
	publicKeys: PublicKey[];
	profile: Profile | undefined;
}

export interface Profile {
	name: string;
	avatar: string | undefined;
}

export interface IUsersClient {
	// Sets the profile for this user
	setProfile(profile: Profile): Promise<void>;
}

export class UsersClient implements IUsersClient {

	async setProfile(profile: Profile): Promise<void> {
		return invoke('set_profile', {
			profile,
		});
	}
}
