<script lang="ts">
	import { getContext } from 'svelte';
	import type { ContactsStore } from 'dash-chat-stores';
	import WaInput from '@awesome.me/webawesome/dist/components/input/input.js';
	import SelectAvatar from '../components/SelectAvatar.svelte';
	import { m } from '$lib/paraglide/messages.js';
	import { Card, Button, ListInput, List } from 'konsta/svelte';

	const contactsStore: ContactsStore = getContext('contacts-store');
	let nickname: string | undefined;
	let avatar: string | undefined;

	async function setProfile() {
		await contactsStore.client.setProfile({
			name: nickname!,
			avatar,
		});
	}
</script>

<Card>
	<div class="column" style="gap: 4px">
		<span class="title">{m.createProfile()}</span>

		<div class="row" style="align-items: center">
			<SelectAvatar bind:value={avatar}></SelectAvatar>

			<List strongIos nested>
				<ListInput
					outline
					label={m.name()}
					oninput={e => {
						nickname = (e.target as HTMLInputElement).value!;
					}}
				></ListInput>
			</List>
		</div>

		<Button onclick={setProfile}>{m.createProfile()}</Button>
	</div>
</Card>
