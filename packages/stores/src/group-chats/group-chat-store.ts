import { invoke } from '@tauri-apps/api/core';
import { ReactivePromise, reactive } from 'signalium';

import { Profile } from '../contacts/contacts-client';
import { ContactsStore } from '../contacts/contacts-store';
import { DevicesStore } from '../devices/devices-store';
import { LogsStore } from '../p2panda/logs-store';
import { SimplifiedOperation } from '../p2panda/simplified-types';
import { AgentId, PublicKey, TopicId } from '../p2panda/types';
import { ChatId, Payload } from '../types';
import { GroupChatClient, Message, MessageContent } from './group-chat-client';

export interface GroupInfo {
	name: string;
	description: string | undefined;
	avatar: string | undefined;
}

export interface GroupMember {
	actorId: AgentId;
	profile: Profile | undefined;
	admin: boolean;
}

export class GroupChatStore {
	constructor(
		protected logsStore: LogsStore<Payload>,
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
		// const allLogs = await this.logsStore.logsForAllAuthors(this.chatId);
		// const messages = await invoke('get_messages', {
		// 	chatId: this.chatId,
		// });
		// const messages : Array<SimplifiedOperation<ChatMessageContent>> = [{
		// 	hash: '',
		// 	header: {

		// 	}
		// }]
		const messages: Array<Message> = [
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
			{
				content: 'heeey',
				author: await this.contactsStore.myAgentId(),
				timestamp: Date.now(),
			},
		];

		return messages;
	});

	membersIds = reactive(async () => {
		const myActorId = await this.contactsStore.myAgentId();
		return [myActorId];
	});

	me = reactive(async () => {
		const actorId = await this.contactsStore.myAgentId();
		return await this.members(actorId);
	});

	allMembers = reactive(async () => {
		const membersIds = await this.membersIds();

		const members = await ReactivePromise.all(
			membersIds.map(memberId => this.members(memberId)),
		);

		const allMembers: Record<AgentId, GroupMember> = {};

		for (let i = 0; i < membersIds.length; i++) {
			allMembers[membersIds[i]] = members[i];
		}

		return allMembers;
	});

	members = reactive(async (actorId: AgentId) => {
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

	sendMessage(content: MessageContent) {
		return this.client.sendMessage(this.chatId, content);
	}
}
