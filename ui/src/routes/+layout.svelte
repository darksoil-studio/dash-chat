<script lang="ts">
	import { setContext } from 'svelte';
	import { UsersStore, type Profile } from '../stores/users-store';
	import { useSignal } from '../signals/svelte-adaptor';
	import { LogsStore } from '../p2panda/logs-store';
	import { LocalStorageLogsClient } from '../stores/mock/client';
	import { LocalStorageUsersClient } from '../stores/mock/users-client';

	const myPubKey =`random`

	const logsClient = new LocalStorageLogsClient(myPubKey);
	const logsStore = new LogsStore(logsClient);

	const usersClient = new LocalStorageUsersClient(logsClient);

	const usersStore = new UsersStore(logsStore, usersClient);
	setContext('users-store', usersStore);

	const me = usersStore.me;
	me.subscribe(console.log)

	setTimeout(()=> {
		logsClient.create(myPubKey, "profile", {
			name: `${Date.now()}`
		} as Profile)
	}, 5000)

</script>

<main class="container">
	{#await $me}
		<span>loading</span>
		{:then me}
		<p>{me?.profile?.name}</p>
	{/await}
	<slot />
</main>

<style>
</style>
