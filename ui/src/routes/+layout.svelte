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
		DevicesClient,
		DevicesStore,
	} from 'dash-chat-stores';

	import SplashscreenPrompt from '../splashscreen/SplashscreenPrompt.svelte';
	import { isMobile } from '../utils/environment';
	import { setupInsets} from '../utils/insets';
	import '../utils/localization';
	import { setLocale } from '$lib/paraglide/runtime';
	// setLocale('es')

	let { children } = $props();

	const logsClient = new TauriLogsClient<TopicId, Payload>();
	const logsStore = new LogsStore<TopicId, Payload>(logsClient);

	const devicesClient = new DevicesClient();
	const devicesStore = new DevicesStore(logsStore, devicesClient);
	setContext('devices-store', devicesStore);

	const contactsClient = new ContactsClient();
	const contactsStore = new ContactsStore(
		logsStore,
		devicesStore,
		contactsClient,
	);
	setContext('contacts-store', contactsStore);

	const chatsClient = new ChatsClient();
	const chatsStore = new ChatsStore(logsStore, contactsStore, chatsClient);
	setContext('chats-store', chatsStore);

	if (isMobile) setupInsets();

</script>

<main class="container column" style="flex: 1">
	<SplashscreenPrompt>
		{@render children()}
	</SplashscreenPrompt>
</main>

<style>
	main {
		margin-top: var(--safe-area-inset-top, 0);
		margin-bottom: var(--safe-area-inset-bottom, 0);
	}
</style>
