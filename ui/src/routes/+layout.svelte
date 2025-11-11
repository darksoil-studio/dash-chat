<script lang="ts">
	import { setContext } from 'svelte';
	import {
		UsersStore,
		LogsStore,
		TauriLogsClient,
		UsersClient,
		type TopicId,
		type Payload,
		FriendsClient,
		FriendsStore
	} from 'dash-chat-stores';
	import SplashscreenPrompt from '../splashscreen/SplashscreenPrompt.svelte';
	let { children } = $props();

	const logsClient = new TauriLogsClient<TopicId, Payload>();
	const logsStore = new LogsStore<TopicId, Payload>(logsClient);

	const usersClient = new UsersClient();
	const usersStore = new UsersStore(logsStore, usersClient);
	setContext('users-store', usersStore);

	const friendsClient= new FriendsClient();
	const friendsStore= new FriendsStore(logsStore, friendsClient);
	setContext('friends-store', friendsStore);
</script>

<main class="container column" style="flex: 1">
	<SplashscreenPrompt>
		{@render children()}
	</SplashscreenPrompt>
</main>

<style>
</style>
