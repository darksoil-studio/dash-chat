export type PublicKey = string;

export type UserId = PublicKey;

export interface User {
	publicKeys: PublicKey[];
	profile: Profile;
}

export interface Profile {
	name: string;
	avatar_src: string | undefined;
}

export interface UsersClient {
	// Get the user public keys and profile for the given user ID
	getUser(userId: UserId): Promise<User | undefined>;
}
