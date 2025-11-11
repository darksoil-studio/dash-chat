import { invoke } from '@tauri-apps/api/core';

import { ActorId, LongTermKeyBundle } from '../p2panda/types';
import { User, UserId } from '../users-client';

export type FriendRequestId = string;

export type Friend = User;
export type FriendId = UserId;

// export type MemberCode = [LongTermKeyBundle, ActorId];
export type MemberCode = string;

export interface IFriendsClient {
	/// Friends

	myMemberCode(): Promise<MemberCode>;

	// Remove friend
	addFriend(memberCode: MemberCode): Promise<void>;

	// Remove friend
	// removeFriend(friend: FriendId): Promise<void>;

	/// Friend Requests

	// // Send friend request to the given user
	// sendFriendRequest(userId: UserId): Promise<void>;

	// // Accept friend request for the given user
	// acceptFriendRequest(userId: UserId): Promise<void>;

	// // Reject friend request for the given user
	// rejectFriendRequest(userId: UserId): Promise<void>;

	// // Cancel friend request for the given user
	// cancelFriendRequest(userId: UserId): Promise<void>;
}

export class FriendsClient implements IFriendsClient {
	myMemberCode(): Promise<MemberCode> {
		return invoke('my_member_code');
	}

	addFriend(memberCode: MemberCode): Promise<void> {
		return invoke('add_friend', {
			memberCode,
		});
	}

	// removeFriend(friendId: FriendId): Promise<void> {
	// 	return invoke('remove_friend', {
	// 		friendId,
	// 	});
	// }
}
