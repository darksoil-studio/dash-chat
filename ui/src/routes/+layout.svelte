<script lang="ts">
	import { setContext } from 'svelte';
	import { UsersStore } from '../stores/users-store';
	import { LogsStore } from '../p2panda/logs-store';
	import { LocalStorageLogsClient } from '../stores/mock/client';
	import {  useSignal } from '../stores/use-signal';
	import { TauriLogsClient } from '../p2panda/tauri-logs-client';
	import { UsersClient, type Profile } from '../stores/users-client';

	// const logsClient = new TauriLogsClient();
	const logsClient = new LocalStorageLogsClient('random');
	const logsStore = new LogsStore(logsClient);

	const usersClient = new UsersClient();
	const usersStore = new UsersStore(logsStore, usersClient);

	setContext('users-store', usersStore);

	const me = useSignal(usersStore.me);
	me.subscribe(v => console.log('hey',v))

	setTimeout(() => {
		logsClient.create("random", 'random', {
			name: `${Date.now()}`,
		} as Profile);
	}, 5000);
</script>

<main class="container">
	{#await $me}
		<span>loading
		</span>
	{:then me}
		{me?.profile?.name}
	{/await}
	<slot />
</main>

<style>
</style>
