<script lang="ts">
	import { setContext } from 'svelte';
	import { UsersStore } from '../stores/users-store';
	import { useSignal } from '../signals/svelte-adaptor';
	import { LogsStore } from '../p2panda/logs-store';
	import { LocalStorageLogsClient } from '../stores/mock/client';
	import { LocalStorageUsersClient } from '../stores/mock/users-client';

	const logsClient = new LocalStorageLogsClient(`${Math.random()}`);
	const logsStore = new LogsStore(logsClient);

	const usersClient = new LocalStorageUsersClient(logsClient);

	const usersStore = new UsersStore(logsStore, usersClient);
	setContext('users-store', usersStore);
</script>

<main class="container">
	{#await usersStore.me.load() then me}
		<p>{$me.profile}</p>
	{/await}
	<slot />
</main>

<style>
</style>
