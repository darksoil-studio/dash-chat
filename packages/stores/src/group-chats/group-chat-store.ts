import { invoke } from '@tauri-apps/api/core';
import { ReactivePromise, reactive } from 'signalium';

import { Profile } from '../contacts/contacts-client';
import { ContactsStore } from '../contacts/contacts-store';
import { DevicesStore } from '../devices/devices-store';
import { LogsStore } from '../p2panda/logs-store';
import { SimplifiedOperation } from '../p2panda/simplified-types';
import { ActorId, PublicKey, TopicId } from '../p2panda/types';
import { ChatId, Payload } from '../types';
import { ChatMessageContent, GroupChatClient } from './group-chat-client';

export interface GroupInfo {
	name: string;
	description: string | undefined;
	avatar: string | undefined;
}

export interface GroupMember {
	actorId: ActorId;
	profile: Profile | undefined;
	admin: boolean;
}

export class GroupChatStore {
	constructor(
		protected logsStore: LogsStore<TopicId, Payload>,
		protected contactsStore: ContactsStore,
		public client: GroupChatClient,
		public chatId: ChatId,
	) {}

	info = reactive(async () => {
		const info: GroupInfo = {
			name: 'mygroup',
			description: 'descmygroup',
			avatar: undefined,
		};
		return info;
	});

	messages = reactive(async () => {
		const allLogs = await this.logsStore.logsForAllAuthors(this.chatId);
		console.log(allLogs);
		// const messages = await invoke('get_messages', {
		// 	chatId: this.chatId,
		// });
		// const messages : Array<SimplifiedOperation<ChatMessageContent>> = [{
		// 	hash: '',
		// 	header: {
				
		// 	}
		// }]
		return [];

		// const messages: Array<SimplifiedOperation<ChatMessageContent>> = [];
		// console.log(allLogs);

		// const setProfiles: Array<[number, Profile]> = log
		// 	.filter(
		// 		l =>
		// 			l.body?.type === 'Announcements' &&
		// 			l.body.payload.type === 'SetProfile',
		// 	)
		// 	.map(l => [
		// 		l.header.timestamp,
		// 		((l.body! as any).payload as AnnouncementPayload).payload,
		// 	]);
	});

	membersIds = reactive(async () => {
		const myActorId = await this.contactsStore.myChatActorId();
		return [myActorId];
	});

	me = reactive(async () => {
		const actorId = await this.contactsStore.myChatActorId();
		return await this.members(actorId);
	});

	allMembers = reactive(async () => {
		const membersIds = await this.membersIds();

		const members = await ReactivePromise.all(
			membersIds.map(memberId => this.members(memberId)),
		);

		const allMembers: Record<ActorId, GroupMember> = {};

		for (let i = 0; i < membersIds.length; i++) {
			allMembers[membersIds[i]] = members[i];
		}

		return members;
	});

	members = reactive(async (actorId: ActorId) => {
		const profile = await this.contactsStore.profiles(actorId);

		const member: GroupMember = {
			actorId,
			profile,
			admin: true,
		};

		return member;
	});

	/// Actions

	addMember(member: PublicKey) {
		return this.client.addMember(this.chatId, member);
	}

	sendMessage(content: ChatMessageContent) {
		return this.client.sendMessage(this.chatId, content);
	}
}
