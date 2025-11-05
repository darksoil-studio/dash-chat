<script lang="ts">
	import { setContext } from 'svelte';
	import {
		UsersStore,
		LogsStore,
		TauriLogsClient,
		UsersClient,
	} from 'dash-chat-stores';
	import { useSignal } from '../stores/use-signal';
	import SplashscreenPrompt from '../splashscreen/SplashscreenPrompt.svelte';

	const logsClient = new TauriLogsClient();
	const logsStore = new LogsStore(logsClient);

	const usersClient = new UsersClient();
	const usersStore = new UsersStore(logsStore, usersClient);

	setContext('users-store', usersStore);
</script>

<main class="container">
	<SplashscreenPrompt>
		<slot>
		</slot>
	</SplashscreenPrompt>
</main>

<style>
</style>
