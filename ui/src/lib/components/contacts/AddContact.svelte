<script lang="ts">
	import { getContext } from 'svelte';
	import type { ContactsStore } from 'dash-chat-stores';
	import { m } from '$lib/paraglide/messages.js';
	import { Card, Button, ListInput, List } from 'konsta/svelte';

	const usersStore: ContactsStore = getContext('contacts-store');
	let nickname = $state<string>('');
	let avatar: string | undefined;

	async function addContact() {
		// await usersStore.client.addContact(nickname!);
	}
</script>

<Card style="margin: 1rem">
	<div class="column" style="gap: 1rem">
		<span class="text-lg font-semibold">{m.addContact()}</span>

		<List nested>
			<ListInput type="text" bind:value={nickname} label={m.name()} />
		</List>

		<Button onClick={addContact}>{m.addContact()}</Button>
	</div>
</Card>
