import { invoke } from '@tauri-apps/api/core';
import { reactive } from 'signalium';

import { LogsStore } from '../p2panda/logs-store';
import { SimplifiedOperation } from '../p2panda/simplified-types';
import { PublicKey, TopicId } from '../p2panda/types';
import { ChatId, Payload } from '../types';
import { ChatMessageContent, GroupChatClient } from './group-chat-client';

export interface GroupInfo {
	name: string;
	description: string | undefined;
	avatar: string | undefined;
}

export class GroupChatStore {
	constructor(
		protected logsStore: LogsStore<TopicId, Payload>,
		protected client: GroupChatClient,
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
		const messages = await invoke('get_messages', {
			chatId: this.chatId,
		});
		return messages;

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

	/// Actions

	addMember(member: PublicKey) {
		return this.client.addMember(this.chatId, member);
	}

	sendMessage(content: ChatMessageContent) {
		return this.client.sendMessage(this.chatId, content);
	}
}
