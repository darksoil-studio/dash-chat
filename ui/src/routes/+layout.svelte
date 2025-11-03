<script lang="ts">
	import { setContext } from 'svelte';
	import { UsersStore } from '../stores/users-store';
	import { LogsStore } from '../p2panda/logs-store';
	import { LocalStorageLogsClient } from '../stores/mock/client';
	import { LocalStorageUsersClient } from '../stores/mock/users-client';
	import {  useSignal } from '../stores/use-signal';
	import { reactive, relay, signal } from 'signalium';
	import { TauriLogsClient } from '../p2panda/tauri-logs-client';
	import { UsersClient } from '../stores/users-client';

	const logsClient = new TauriLogsClient();
	const logsStore = new LogsStore(logsClient);

	const usersClient = new UsersClient();
	const usersStore = new UsersStore(logsStore, usersClient);

	setContext('users-store', usersStore);

	// const counter =reactive(()=>relay<number>(state => {
	// // state.value = 0
	// 	console.log('hi')

	// 	const id = setInterval(
	// 		() => state.value = state.value != undefined ? state.value + 1 :0,
	// 		1000,
	// 	);

	// 	return () => clearInterval(id);
	// }));

	// const r = reactive(async ()=>counter())

	// const me = useSignal(r);
	const me = useSignal(usersStore.me);
	// me.subscribe(v => console.log('aa', v));

	// setTimeout(() => {
	// 	logsClient.create(myPubKey, 'profile', {
	// 		name: `${Date.now()}`,
	// 	} as Profile);
	// }, 5000);
</script>

<main class="container">
	{#await $me}
		<span>loading
		</span>
	{:then i} 
		{i}
	{/await}
	<slot />
</main>

<style>
</style>
