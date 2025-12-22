<script lang="ts">
	import '@awesome.me/webawesome/dist/styles/webawesome.css';

	import '../app.css';
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
	import { App, KonstaProvider } from 'konsta/svelte';

	import SplashscreenPrompt from '../splashscreen/SplashscreenPrompt.svelte';
	import { isMobile } from '../utils/environment';
	import { setupInsets } from '../utils/insets';

	import { setLocale } from '$lib/paraglide/runtime';
	// setLocale('en');

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
	let theme: 'ios' | 'material' = 'ios';
</script>

<KonstaProvider {theme}>
	<App {theme} class={`k-${theme}`}>
		<main style="height: 100vh; width: 100vw; display: flex">
			<SplashscreenPrompt>
				{@render children()}
			</SplashscreenPrompt>
		</main>
	</App>
</KonstaProvider>

<style>
	main {
		margin-top: var(--safe-area-inset-top, 0);
		margin-bottom: var(--safe-area-inset-bottom, 0);
	}
</style>
