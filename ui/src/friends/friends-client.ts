import { User, UserId } from "../users/user-client";

export type FriendRequestId = string;

export type UnsubscribeFn = () => void;

export type Friend = User;
export type FriendId = UserId;

export interface FriendsClient {
	/// Friends

	// Get all the current friends
	getAllFriends(): Promise<Array<Friend>>;

	// Executes the handler callback whenever a new friend has been added
	onFriendAdded(handler: (newFriend: FriendId) => void): UnsubscribeFn;

	// Executes the handler callback whenever a new friend has been added
	onFriendRemoved(handler: (removedFriend: FriendId) => void): UnsubscribeFn;

	// Remove friend
	removeFriend(friend: FriendId): Promise<void>;

	/// Friend Requests

	// Get all the current friends
	getPendingFriendRequests(): Promise<Array<User>>;

	// Executes the handler callback whenever a new friend request is received
	onFriendRequestReceived(handler: (userId: UserId) => void): UnsubscribeFn;

	// Executes the handler callback whenever a friend request is rejected
	onFriendRequestRejected(handler: (userId: UserId) => void): UnsubscribeFn;

	// Executes the handler callback whenever a friend request is cancelled
	onFriendRequestCancelled(handler: (userId: UserId) => void): UnsubscribeFn;

	// Send friend request to the given user
	sendFriendRequest(userId: UserId): Promise<void>;

	// Accept friend request for the given user
	acceptFriendRequest(userId: UserId): Promise<void>;

	// Reject friend request for the given user
	rejectFriendRequest(userId: UserId): Promise<void>;

	// Cancel friend request for the given user
	cancelFriendRequest(userId: UserId): Promise<void>;
}
