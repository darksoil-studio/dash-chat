import { type PublicKey, type TopicId } from '../p2panda/types';

export type UserId = PublicKey;

export interface User {
	publicKeys: PublicKey[];
	profile: Profile | undefined;
}

export interface Profile {
	name: string;
	avatar_src: string | undefined;
}

export interface IUsersClient {
	// Sets the profile for this user
	setProfile(profile: Profile): Promise<void>;
}

export class UsersClient implements IUsersClient {
	constructor() {}

	async setProfile(profile: Profile): Promise<void> {
		// TODO: call the `set_profile` command
	}
}
