import { toPromise } from '@darksoil-studio/holochain-signals';
import {
	MessengerClient,
	MessengerStore,
} from '@darksoil-studio/messenger-zome';
import { CellId, ProvisionedCell } from '@holochain/client';
import {
	AgentApp,
	enableAndGetAgentApp,
	pause,
	runScenario,
} from '@holochain/tryorama';
import { assert, test } from 'vitest';

import { appPath, oldAppPath } from './app-path.js';
import { setup } from './setup.js';

test('migrate from the previous happ to the new version, assert that the data is maintainted', async () => {
	await runScenario(async scenario => {
		const [alice, bob] = await setup(scenario, 2, oldAppPath);

		const peerChat = await alice.messengerStore.client.createPeerChat(
			bob.player.agentPubKey,
		);

		await alice.messengerStore.peerChats.get(peerChat).sendMessage({
			message: 'hey!',
			reply_to: undefined,
		});

		await pause(5000);

		const appInfo = await alice.player.conductor.installApp({
			appBundleSource: {
				type: 'path',
				value: appPath,
			},
			options: {
				agentPubKey: alice.player.cells[0].cell_id[1],
			},
		});
		const port = await alice.player.conductor.attachAppInterface();
		const issued = await alice.player.conductor
			.adminWs()
			.issueAppAuthenticationToken({
				installed_app_id: appInfo.installed_app_id,
			});
		const appWs = await alice.player.conductor.connectAppWs(issued.token, port);
		const agentApp: AgentApp = await enableAndGetAgentApp(
			alice.player.conductor.adminWs(),
			appWs,
			appInfo,
		);

		const previousAppInfo = await alice.messengerStore.client.client.appInfo();

		assert.equal('provisioned', previousAppInfo.cell_info['main'][0].type);
		const previousCellId: CellId = (
			previousAppInfo.cell_info['main'][0].value as ProvisionedCell
		).cell_id;

		await appWs.callZome({
			role_name: 'main',
			zome_name: 'messenger',
			payload: previousCellId,
			fn_name: 'migrate_from_old_cell',
		});

		await appWs.callZome({
			role_name: 'main',
			zome_name: 'friends',
			payload: previousCellId,
			fn_name: 'migrate_from_old_cell',
		});

		await pause(200);

		const aliceStore2 = new MessengerStore(new MessengerClient(appWs, 'main'));

		const peerChatStore = aliceStore2.peerChats.get(peerChat);
		const messages = await toPromise(peerChatStore.messages);
		assert.equal(Object.keys(messages).length, 1);
	});
});
