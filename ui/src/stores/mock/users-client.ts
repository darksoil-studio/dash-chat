import type { IUsersClient, Profile } from '../users-store';
import type { LocalStorageLogsClient } from './client';

export class LocalStorageUsersClient implements IUsersClient {
	constructor(protected client: LocalStorageLogsClient) {}

	async setProfile(profile: Profile): Promise<void> {
		const myPubKey = await this.client.myPubKey();
		this.client.create(myPubKey, 'profile', profile);
	}
}
