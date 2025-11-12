<script lang="ts">
	import '@awesome.me/webawesome/dist/components/input/input.js';
	import '@awesome.me/webawesome/dist/components/card/card.js';
	import '@awesome.me/webawesome/dist/components/button/button.js';
	import { getContext } from 'svelte';
	import type { ContactsStore } from 'dash-chat-stores';
	import WaInput from '@awesome.me/webawesome/dist/components/input/input.js';
	import '@darksoil-studio/holochain-elements/dist/elements/select-avatar.js';
	import SelectAvatar from '../components/SelectAvatar.svelte';

	const contactsStore: ContactsStore = getContext('contacts-store');
	let nickname: string | undefined;
	let avatar: string | undefined

	async function setProfile() {
		await contactsStore.client.setProfile({
			name: nickname!,
			avatar,
		});
	}
</script>

<wa-card>
	<div class="column" style="gap: var(--wa-space-m)">

		<span class="title">Create Profile</span>

		<div class="row" style="gap: var(--wa-space-xs)">
			<SelectAvatar bind:avatar={avatar}>
			</SelectAvatar>

			<wa-input
				oninput={(e: CustomEvent) => {
					nickname = (e.target as WaInput).value!;
				}}
			>
			</wa-input>
		</div>

		<wa-button onclick={setProfile}>Create Profile </wa-button>
	</div>
</wa-card>
