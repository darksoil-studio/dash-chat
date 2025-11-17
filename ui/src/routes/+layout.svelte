<script lang="ts">
	import '@awesome.me/webawesome/dist/styles/webawesome.css';
	import { setContext } from 'svelte';
	import {
		ChatsClient,
		ChatsStore,
		LogsStore,
		TauriLogsClient,
		type TopicId,
		type Payload,
		ContactsClient,
		ContactsStore,
	} from 'dash-chat-stores';
	import SplashscreenPrompt from '../splashscreen/SplashscreenPrompt.svelte';
	let { children } = $props();

	const logsClient = new TauriLogsClient<TopicId, Payload>();
	const logsStore = new LogsStore<TopicId, Payload>(logsClient);

	const contactsClient = new ContactsClient();
	const contactsStore = new ContactsStore(logsStore, contactsClient);
	setContext('contacts-store', contactsStore);

	const chatsClient = new ChatsClient();
	const chatsStore = new ChatsStore(logsStore, chatsClient);
	setContext('chats-store', chatsStore);
</script>

<main class="container column" style="flex: 1">
	<SplashscreenPrompt>
		{@render children()}
	</SplashscreenPrompt>
</main>

<style>
</style>
