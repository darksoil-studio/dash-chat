<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/avatar/avatar.js';
	import { m, members } from '$lib/paraglide/messages.js';

	import { getContext } from 'svelte';
	import type { ContactsStore, ChatsStore, PublicKey } from 'dash-chat-stores';
	import { wrapPathInSvg } from '@darksoil-studio/holochain-elements';
	import { mdiAccountMultiplePlus } from '@mdi/js';

	import { useReactivePromise } from '../../stores/use-signal';
	import Avatar from '../../components/Avatar.svelte';
	import SelectAvatar from '../../components/SelectAvatar.svelte';
	import {
		Page,
		Navbar,
		NavbarBackLink,
		Button,
		Card,
		Link,
		List,
		ListItem,
		Checkbox,
		ListInput,
		BlockTitle,
		Preloader,
	} from 'konsta/svelte';
	import { mdiArrowNext } from '../../utils/icon';

	const contactsStore: ContactsStore = getContext('contacts-store');
	const chatsStore: ChatsStore = getContext('chats-store');

	const contacts = useReactivePromise(contactsStore.profilesForAllContacts);

	let currentPage: 'members' | 'group-info' = $state('members');
	let selectedContacts: Set<PublicKey> = new Set();
	let groupName = $state('');
	let groupAvatar = $state<string | undefined>(undefined);

	async function createGroupChat() {
		const contacts = Array.from(selectedContacts);
		const groupStore = await chatsStore.createGroup(contacts);
		window.location.href = `/group-chat/${groupStore.chatId}`;
	}
</script>

{#if currentPage === 'members'}
	<Page>
		<Navbar title={m.newGroup()}>
			{#snippet left()}
				<NavbarBackLink onClick={() => window.history.back()} />
			{/snippet}

			{#snippet right()}
				<Link onClick={() => (currentPage = 'group-info')}>
					{m.next()}
					<wa-icon src={wrapPathInSvg(mdiArrowNext)}> </wa-icon>
				</Link>
			{/snippet}
		</Navbar>

		<div class="column" style="flex: 1">
			<div class="center-in-desktop">
				<BlockTitle>{m.members()}</BlockTitle>

				<List strong>
					{#await $contacts}
						<div
							class="column"
							style="flex: 1; align-items: center; justify-content: center"
						>
							<Preloader />
						</div>
					{:then contacts}
						{#each contacts as [publicKey, profile]}
							<ListItem label title={profile.name}>
								{#snippet media()}
									<div class="row gap-3" style="align-items: center">
										<Checkbox
											checked={selectedContacts.has(publicKey)}
											onChange={e => {
												const target = e.target as HTMLInputElement;
												if (target.checked) {
													selectedContacts.add(publicKey);
												} else {
													selectedContacts.delete(publicKey);
												}
												selectedContacts = selectedContacts;
											}}
										/>
										<Avatar chatActorId={publicKey}></Avatar>
									</div>
								{/snippet}
							</ListItem>
						{:else}
							<ListItem title={m.noContactsYet()} />
						{/each}
					{/await}
				</List>
			</div>
		</div></Page
	>
{:else}
	<Page>
		<Navbar title={m.newGroup()}>
			{#snippet left()}
				<NavbarBackLink onClick={() => (currentPage = 'members')} />
			{/snippet}

			{#snippet right()}
				<Link onClick={createGroupChat}>
					<wa-icon src={wrapPathInSvg(mdiAccountMultiplePlus)}> </wa-icon>
					{m.createGroup()}
				</Link>
			{/snippet}
		</Navbar>

		<div class="column" style="flex: 1">
			<div class="center-in-desktop m-1">
				<BlockTitle>{m.groupInfo()}</BlockTitle>
				<Card raised>
					<div class="column gap-2">
						<div class="row" style="align-items: center">
							<SelectAvatar bind:value={groupAvatar}></SelectAvatar>

							<List nested>
								<ListInput
									type="text"
									outline
									bind:value={groupName}
									label={m.name()}
								/>
							</List>
						</div>
					</div>
				</Card>
			</div>
		</div></Page
	>
{/if}
