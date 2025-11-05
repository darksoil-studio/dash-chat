import { MemoMap } from "@darksoil-studio/holochain-utils";
import { UnsubscribeFn } from "../friends/friends-client";

import {listen} from '@tauri-apps/api/event'

import Emittery from "emittery";
import { signal } from "alien-signals";

export type PublicKey = string;

export type UserId = PublicKey;

export interface User {
	publicKeys: PublicKey[];
	profile: Profile;
}

export interface Profile {
	name: string;
	avatar: string | undefined;
}

export interface UsersClient {
	// Gets my user
	me(): Promise<User>;

	// Sets the profile for this user
	setProfile(profile: Profile): Promise<void>;

	onNewUserProfile(handler: (userId: UserId, profile: Profile) => unknown): UnsubscribeFn;

	onNewUserPublicKey(handler: (userId: UserId, publicKey: PublicKey) => unknown): UnsubscribeFn;
	
	// Get the user public keys and profile for the given user ID
	getUser(userId: UserId): Promise<User | undefined>;
}

// export class UserAvatar {
// 	userId: UserId;
// 	store: UsersStore

// 	render() {
// 		this.store.user.get(this.userId).status;
// 		return html``
// 	}
// }

// export class TauriUsersClient implements UsersClient {

// onNewUserProfile(handler: (userId: UserId, profile: Profile) => unknown): UnsubscribeFn {
// 		listen('new-user-profile', profile => )
// }
// }

export type AsyncResult<T> = {
	status: 'initial'
} |  {
	status: 'loading'
} | {
	status: 'completed';
	value: T
} | {
	status: 'error';
	error: unknown
}

function relay<T>(fn: (set: ((v: T) => void), get: (()=>AsyncResult<T>)) => Promise<void>): () => AsyncResult<T> {
	const s = signal<AsyncResult<T>>({
		status: 'initial'
	});

	return () => {
		if (s().status === 'initial') {
			s({
				status: 'loading'
			})
			fn(value => {
				s({
					status: 'completed',
					value
				})
			}, () => s()).catch(error => s({
				status: 'error',
				error
			}))
		}

		return s()
	};
}

export class UsersStore {
	// usersProfiles= new Signal.State<Record<string, Profile>>({});
	
	constructor(protected client: UsersClient) {}

	me = relay(set => {
		
	})

	// Returns whether this two user ids are for the same user
	//
	// This is useful in case of linked devices where the user
	// will have multiple public keys, and any one of them identifies
	// the user.
	isSameUser(userId1: UserId, userId2: UserId ): AsyncComputed<boolean> {}

	user = new MemoMap((userId:UserId) => new AsyncComputed(() => ));
}
