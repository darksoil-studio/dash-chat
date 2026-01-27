<script lang="ts">
	import '@awesome.me/webawesome/dist/styles/webawesome.css';

	import '../app.css';
	import { setContext, onMount } from 'svelte';
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
	import { App, KonstaProvider, Toast } from 'konsta/svelte';

	import SplashscreenPrompt from '$lib/components/splashscreen/SplashscreenPrompt.svelte';
	import { TOAST_TTL_MS, type ToastEvent } from '$lib/utils/toasts';

	import { setLocale } from '$lib/paraglide/runtime';
	setLocale('en');

	let { children } = $props();

	const logsClient = new TauriLogsClient<TopicId, Payload>();
	const logsStore = new LogsStore<Payload>(logsClient);

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

	let theme: 'ios' | 'material' = 'material';

	let toastOpen = $state(false);
	let toastMessage = $state('');
	let toastVariant = $state<'default' | 'error'>('default');
	let toastTimeout: ReturnType<typeof setTimeout> | undefined;

	function handleToast(event: CustomEvent<ToastEvent>) {
		clearTimeout(toastTimeout);
		toastMessage = event.detail.message;
		toastVariant = event.detail.variant ?? 'default';
		toastOpen = true;
		toastTimeout = setTimeout(() => {
			toastOpen = false;
		}, TOAST_TTL_MS);
	}

	onMount(() => {
		window.addEventListener('app:toast', handleToast as EventListener);
		return () => {
			window.removeEventListener('app:toast', handleToast as EventListener);
			clearTimeout(toastTimeout);
		};
	});
</script>

<KonstaProvider {theme}>
	<App safeAreas {theme} class={`k-${theme}`}>
		<SplashscreenPrompt>
			{@render children()}
		</SplashscreenPrompt>
		<Toast
			position="center"
			class={toastVariant === 'error' ? 'k-color-brand-red' : ''}
			opened={toastOpen}
		>
			{toastMessage}
		</Toast>
	</App>
</KonstaProvider>
