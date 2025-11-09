<script lang="ts">
	import '@awesome.me/webawesome/dist/components/input/input.js';
	import '@awesome.me/webawesome/dist/components/card/card.js';
	import '@awesome.me/webawesome/dist/components/button/button.js';
	import { getContext } from 'svelte';
	import type { UsersStore } from 'dash-chat-stores';
	import WaInput from '@awesome.me/webawesome/dist/components/input/input.js';
	import '@darksoil-studio/holochain-elements/dist/elements/select-avatar.js';

	const usersStore: UsersStore = getContext('users-store');
	let nickname: string | undefined;
	let avatar: string | undefined;

	async function addFriend() {
		// await usersStore.client.addFriend(nickname!);
	}
</script>

<wa-card>
	<div class="column" style="gap: var(--wa-space-m)">
		<span class="title">Add Friend</span>

		<div class="row" style="gap: var(--wa-space-xs)">
			<input type="text" bind:value={nickname} />
		</div>

		<wa-button onclick={addFriend}>Add Friend </wa-button>
	</div>
</wa-card>
