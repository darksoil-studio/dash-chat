<script lang="ts">
	import { splashscreenDismissed } from './utils';
	import '@awesome.me/webawesome/dist/components/input/input.js';
	import '@awesome.me/webawesome/dist/components/button/button.js';
	import { getContext } from 'svelte';
	import type { UsersStore } from 'dash-chat-stores';
	import WaInput from '@awesome.me/webawesome/dist/components/input/input.js';
	import { useSignal } from '../stores/use-signal';

	const usersStore: UsersStore = getContext('users-store');
	let nickname: string | undefined;

	async function setProfile() {
		await usersStore.client.setProfile({
			name: nickname!,
			avatar: undefined,
		});

		splashscreenDismissed.dismiss();
	}
	
	const me = useSignal(usersStore.me)
</script>

<wa-input
	oninput={(e: CustomEvent) => {
		nickname = (e.target as WaInput).value!;
		console.log(nickname);
	}}
>
</wa-input>
<wa-button onclick={setProfile}>Create Profile </wa-button>
