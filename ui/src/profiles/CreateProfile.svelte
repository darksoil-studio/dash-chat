<script lang="ts">
	import { getContext } from 'svelte';
	import type { ContactsStore } from 'dash-chat-stores';
	import SelectAvatar from '../components/SelectAvatar.svelte';
	import { m } from '$lib/paraglide/messages.js';
	import { Card, Button, ListInput, List, BlockTitle } from 'konsta/svelte';

	const contactsStore: ContactsStore = getContext('contacts-store');
	let nickname = $state<string>('');
	let avatar= $state<string | undefined>(undefined);

	async function setProfile() {
		await contactsStore.client.setProfile({
			name: nickname!,
			avatar,
		});
	}
</script>

<Card raised>
	<div class="column gap-2">
		<span class="title">{m.createProfile()}</span>
		<div class="row gap-1" style="align-items: center;">
			<SelectAvatar bind:value={avatar}></SelectAvatar>
			<List nested>
				<ListInput
					outline
					type="text"
					bind:value={nickname}
					label={m.name()}
				/>
			</List>
		</div>

		<Button onClick={setProfile}>{m.createProfile()}</Button>
	</div>
</Card>
