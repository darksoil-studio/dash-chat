import type { User, UserId } from "./users-store";
import type { AsyncSignal } from "../signals/async-computed";

export type FriendRequestId = string;

export type Friend = User;
export type FriendId = UserId;

export interface FriendsClient {
	/// Friends

	// Get all the current friends
	allFriends(): AsyncSignal<Array<Friend>>;

	// Remove friend
	removeFriend(friend: FriendId): Promise<void>;

	/// Friend Requests

	// Get all the current friends
	pendingFriendRequests(): AsyncSignal<Array<User>>;

	// Send friend request to the given user
	sendFriendRequest(userId: UserId): Promise<void>;

	// Accept friend request for the given user
	acceptFriendRequest(userId: UserId): Promise<void>;

	// Reject friend request for the given user
	rejectFriendRequest(userId: UserId): Promise<void>;

	// Cancel friend request for the given user
	cancelFriendRequest(userId: UserId): Promise<void>;
}
