<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/button/button.js';
	import '@awesome.me/webawesome/dist/components/checkbox/checkbox.js';

	import { getContext } from 'svelte';
	import type { ContactsStore, ChatsStore, PublicKey } from 'dash-chat-stores';
	import { wrapPathInSvg } from '@darksoil-studio/holochain-elements';
	import { mdiAccountMultiplePlus, mdiArrowLeft, mdiArrowRight } from '@mdi/js';

	import { useReactivePromise } from '../../stores/use-signal';
	import Avatar from '../../components/Avatar.svelte';
	import SelectAvatar from '../../components/SelectAvatar.svelte';

	const contactsStore: ContactsStore = getContext('contacts-store');
	const chatsStore: ChatsStore = getContext('chats-store');

	const contacts = useReactivePromise(contactsStore.profilesForAllContacts);

	let currentPage: 'members' | 'group-info' = $state('members');

	let selectedContacts: Set<PublicKey> = new Set();

	async function createGroupChat() {
		const contacts = Array.from(selectedContacts);

		const groupStore = await chatsStore.createGroup(contacts);

		window.location.href = `/group-chat/${groupStore.chatId}`;
	}
</script>

{#if currentPage === 'members'}
	<div class="top-bar">
		<wa-button
			class="circle"
			appearance="plain"
			onclick={() => window.history.back()}
		>
			<wa-icon src={wrapPathInSvg(mdiArrowLeft)}> </wa-icon>
		</wa-button>
		<span class="title" style="flex: 1">New group </span>

		<wa-button
			appearance="plain"
			onclick={() => {
				currentPage = 'group-info';
				console.log('aa', currentPage)

			}}
		>
			Next
			<wa-icon slot="end" src={wrapPathInSvg(mdiArrowRight)}> </wa-icon>
		</wa-button>
	</div>

	<wa-card class="center-in-desktop">
		<div class="column" style="gap: var(--wa-space-m)">
			<span class="title">Members</span>
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
								<Avatar chatActorId={publicKey}></Avatar>

								{profile.name}
							</div>
						</wa-checkbox>
					{:else}
						You don't have any contacts yet.
					{/each}
				</div>
			{/await}
		</div>
	</wa-card>
{:else}
	<div class="top-bar">
		<wa-button
			class="circle"
			appearance="plain"
			onclick={() => {
				currentPage = 'members';
			}}
		>
			<wa-icon src={wrapPathInSvg(mdiArrowLeft)}> </wa-icon>
		</wa-button>
		<span class="title" style="flex: 1">New group</span>

		<wa-button appearance="plain" onclick={createGroupChat}>
			<wa-icon slot="start" src={wrapPathInSvg(mdiAccountMultiplePlus)}>
			</wa-icon>
			Create group
		</wa-button>
	</div>

	<wa-card class="center-in-desktop">
		<div class="column" style="gap: var(--wa-space-m)">
			<span class="title">Group Info </span>

			<div class="row" style="gap: var(--wa-space-s); align-items: center">
				<SelectAvatar></SelectAvatar>

				<wa-input placeholder="Name"> </wa-input>
			</div>
		</div>
	</wa-card>
{/if}

<style>
	wa-checkbox::part(base) {
		align-items: center;
	}
</style>
