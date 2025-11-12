<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/button/button.js';
	import '@awesome.me/webawesome/dist/components/checkbox/checkbox.js';

	import { useReactivePromise } from '../../stores/use-signal';
	import { getContext } from 'svelte';
	import type { ContactsStore, ChatsStore, PublicKey } from 'dash-chat-stores';
	import Avatar from '../../components/Avatar.svelte';
	import { wrapPathInSvg } from '@darksoil-studio/holochain-elements';
	import { mdiAccountMultiplePlus, mdiArrowLeft } from '@mdi/js';

	const contactsStore: ContactsStore = getContext('contacts-store');
	const chatsStore: ChatsStore = getContext('chats-store');

	const contacts = useReactivePromise(contactsStore.profilesForAllContacts);

	let selectedContacts: Set<PublicKey> = new Set();

	async function createGroup() {
		const contacts = Array.from(selectedContacts);

		const groupStore = await chatsStore.createGroup(contacts);
		console.log('hey', contacts);

		window.location.href = `/group-chat/${groupStore.chatId}`;
	}
</script>

<div class="top-bar">
	<wa-button
		class="circle"
		appearance="plain"
		onclick={() => window.history.back()}
	>
		<wa-icon src={wrapPathInSvg(mdiArrowLeft)}> </wa-icon>
	</wa-button>
	<span class="title" style="flex: 1">New group </span>

	<wa-button appearance="plain" onclick={createGroup}>
		<wa-icon slot="start" src={wrapPathInSvg(mdiAccountMultiplePlus)}>
		</wa-icon>
		Create group
	</wa-button>
</div>

<div class="column center-in-desktop" style="gap: var(--wa-space-l)">
	{#await $contacts then contacts}
		<div class="column" style="gap: var(--wa-space-m)">
			{#each contacts as [publicKey, profile]}
				<wa-checkbox
					onchange={(e: Event) => {
						if ((e.target! as any).checked) {
							selectedContacts.add(publicKey);
						} else {
							selectedContacts.delete(publicKey);
						}
					}}
				>
					<div
						class="row"
						style="gap: var(--wa-space-s); align-items: center; margin-left: var(--wa-space-xs)"
					>
						<Avatar {publicKey}></Avatar>

						{profile.name}
					</div>
				</wa-checkbox>
			{:else}
				You don't have any contacts yet.
			{/each}
		</div>
	{/await}
</div>

<style>
	wa-checkbox::part(base) {
		align-items: center;
	}
</style>
