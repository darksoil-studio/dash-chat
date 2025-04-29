import { FriendsClient, FriendsStore } from '@darksoil-studio/friends-zome';
import { Signal, joinAsync } from '@darksoil-studio/holochain-signals';
import {
	MessengerClient,
	MessengerStore,
} from '@darksoil-studio/messenger-zome';
import { HoloHashB64 } from '@holochain/client';
import { Scenario, dhtSync, pause } from '@holochain/tryorama';
import { dirname } from 'path';
import { fileURLToPath } from 'url';

import { appPath } from './app-path.js';

async function addPlayer(scenario: Scenario, appPath: string) {
	const player = await scenario.addPlayerWithApp({
		type: 'path',
		value: appPath,
	});
	await player.conductor
		.adminWs()
		.authorizeSigningCredentials(player.cells[0].cell_id);

	const messengerStore = new MessengerStore(
		new MessengerClient(player.appWs as any, 'main'),
		// new LinkedDevicesStore(
		// 	new LinkedDevicesClient(player.appWs as any, 'main'),
		// ),
		// new ProfilesStore(new ProfilesClient(player.appWs as any, 'main')),
	);
	const friendsStore = new FriendsStore(
		new FriendsClient(player.appWs as any, 'main'),
	);
	await messengerStore.client.queryPrivateEventEntries();

	return {
		player,
		messengerStore,
		friendsStore,
		startUp: async () => {
			await player.conductor.startUp();
			const port = await player.conductor.attachAppInterface();
			const issued = await player.conductor
				.adminWs()
				.issueAppAuthenticationToken({
					installed_app_id: player.appId,
				});
			const appWs = await player.conductor.connectAppWs(issued.token, port);
			// store.client.client = appWs;
			// store.linkedDevicesStore.client.client = appWs;
			// (store.profilesProvider as ProfilesStore).client.client = appWs;
		},
	};
}

async function promiseAllSequential<T>(
	fns: Array<() => Promise<T>>,
): Promise<Array<T>> {
	const results: Array<T> = [];
	for (const fn of fns) {
		results.push(await fn());
	}
	return results;
}

export async function setup(
	scenario: Scenario,
	numPlayers = 2,
	appBundlePath: string = appPath,
) {
	const players = await promiseAllSequential(
		Array.from(new Array(numPlayers)).map(
			() => () => addPlayer(scenario, appBundlePath),
		),
	);

	// Shortcut peer discovery through gossip and register all agents in every
	// conductor of the scenario.
	await scenario.shareAllAgents();

	await dhtSync(
		players.map(p => p.player),
		players[0].player.cells[0].cell_id[0],
	);

	console.log('Setup completed!');

	return players;
}

export async function waitUntil(
	condition: () => Promise<boolean>,
	timeout: number,
) {
	const start = Date.now();
	const isDone = await condition();
	if (isDone) return;
	if (timeout <= 0) throw new Error('timeout');
	await pause(1000);
	return waitUntil(condition, timeout - (Date.now() - start));
}
