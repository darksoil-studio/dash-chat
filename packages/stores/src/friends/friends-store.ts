import { reactive } from 'signalium';

import { LogsStore } from '../p2panda/logs-store';
import { TopicId } from '../p2panda/types';
import { Payload } from '../types';
import { IFriendsClient } from './friends-client';

export class FriendsStore {
	constructor(
		protected logsStore: LogsStore<TopicId, Payload>,
		public client: IFriendsClient,
	) {}

	myMemberCode = reactive(async () => this.client.myMemberCode());
}
