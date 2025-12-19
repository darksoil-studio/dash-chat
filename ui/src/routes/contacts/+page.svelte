<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/button/button.js';
	import '@awesome.me/webawesome/dist/components/avatar/avatar.js';
	import { useReactivePromise } from '../../stores/use-signal';
	import { getContext } from 'svelte';
	import type { ContactRequestId, ContactsStore } from 'dash-chat-stores';
	import Avatar from '../../components/Avatar.svelte';
	import { wrapPathInSvg } from '@darksoil-studio/holochain-elements';
	import { mdiAccountPlus } from '@mdi/js';
	import WaButton from '@awesome.me/webawesome/dist/components/button/button.js';
	import { m, myContacts } from '$lib/paraglide/messages.js';
	import { mdiArrowBack } from '../../utils/icon';
	import Page from '../+page.svelte';
	import {
		BlockTitle,
		Card,
		Link,
		List,
		ListItem,
		Navbar,
		NavbarBackLink,
		Preloader,
	} from 'konsta/svelte';

	const contactsStore: ContactsStore = getContext('contacts-store');

	const incomingContactRequests = useReactivePromise(
		contactsStore.incomingContactRequests,
	);
	const contacts = useReactivePromise(contactsStore.profilesForAllContacts);

	function rejectContactRequest(contactRequestId: ContactRequestId) {}

	function acceptContactRequest(contactRequestId: ContactRequestId) {}
</script>

<Page>
	<Navbar title={m.myContacts()}>
		{#snippet left()}
			<NavbarBackLink onClick={() => (window.location.href = '/')} />
		{/snippet}

		{#snippet right()}
			<Link href="/add-contact" iconOnly>
				<wa-icon src={wrapPathInSvg(mdiAccountPlus)}> </wa-icon>
			</Link>
		{/snippet}
	</Navbar>

	{#await $incomingContactRequests}
		<div
			class="column"
			style="height: 100%; align-items: center; justify-content: center"
		>
			<Preloader />
		</div>
	{:then incomingContactRequests}
		{#if incomingContactRequests.length > 0}
			<BlockTitle>{m.contactRequests()}</BlockTitle>
			<List strong outline inset>
				{#each incomingContactRequests as incomingContactRequest}
					<ListItem title={incomingContactRequest.profile.name}>
						{#snippet media()}
							<wa-avatar
								image={incomingContactRequest.profile.avatar}
								initials={incomingContactRequest.profile.name.slice(0, 2)}
							>
							</wa-avatar>
						{/snippet}
						{#snippet after()}
							<wa-button
								variant="danger"
								onclick={async (e: Event) => {
									const button = e.target as WaButton;
									button.loading = true;

									try {
										await rejectContactRequest(
											incomingContactRequest.contactRequestId,
										);
									} catch (e) {}

									button.loading = false;
								}}
								>{m.reject()}
							</wa-button>

							<wa-button
								variant="brand"
								onclick={async (e: Event) => {
									const button = e.target as WaButton;
									button.loading = true;

									try {
										await acceptContactRequest(
											incomingContactRequest.contactRequestId,
										);
									} catch (e) {}

									button.loading = false;
								}}
								>{m.accept()}
							</wa-button>
						{/snippet}
					</ListItem>
				{/each}
			</List>
		{/if}
	{/await}

	{#await $contacts}
		<div
			class="column"
			style="height: 100%; align-items: center; justify-content: center"
		>
			<Preloader />
		</div>
	{:then contacts}
		<wa-card class="center-in-desktop" style="margin: var(--wa-space-m)">
			<div class="column" style="gap: var(--wa-space-m)">
				{#each contacts as [actorId, profile]}
					<wa-button
						appearance="plain"
						class="fill button-with-avatar"
						href={`/direct-messages/${actorId}`}
					>
						<wa-avatar
							slot="start"
							image={profile.avatar}
							initials={profile.name.slice(0, 2)}
						>
						</wa-avatar>

						{profile.name}
					</wa-button>
				{:else}<span>{m.noContactsYet()}</span>
				{/each}
			</div>
		</wa-card>
	{/await}
</Page>
