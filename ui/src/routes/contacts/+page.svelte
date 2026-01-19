<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/avatar/avatar.js';
	import { useReactivePromise } from '$lib/stores/use-signal';
	import { getContext } from 'svelte';
	import { goto } from '$app/navigation';
	import type { ContactRequestId, ContactsStore } from 'dash-chat-stores';
	import Avatar from '$lib/components/profiles/Avatar.svelte';
	import { mdiAccountPlus } from '@mdi/js';
	import { m, myContacts } from '$lib/paraglide/messages.js';
	import {
		Page,
		BlockTitle,
		Button,
		Link,
		List,
		ListItem,
		Navbar,
		NavbarBackLink,
		Preloader,
	} from 'konsta/svelte';
	import { wrapPathInSvg } from '$lib/utils/icon';

	const contactsStore: ContactsStore = getContext('contacts-store');

	const incomingContactRequests = useReactivePromise(
		contactsStore.incomingContactRequests,
	);
	const contacts = useReactivePromise(contactsStore.profilesForAllContacts);

	async function rejectContactRequest(contactRequestId: ContactRequestId) {
		try {
			// Actual rejection logic here
		} finally {
		}
	}

	async function acceptContactRequest(contactRequestId: ContactRequestId) {
		try {
			// Actual acceptance logic here
		} finally {
		}
	}
</script>

<Page>
	<Navbar title={m.myContacts()} titleClass="opacity1" transparent={true}>
		{#snippet left()}
			<NavbarBackLink onClick={() => goto('/')} />
		{/snippet}

		{#snippet right()}
			<Link href="/add-contact" iconOnly>
				<wa-icon src={wrapPathInSvg(mdiAccountPlus)}> </wa-icon>
			</Link>
		{/snippet}
	</Navbar>

	<div class="column" style="flex: 1">
		<div class="center-in-desktop">
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
					<List strongIos insetIos>
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
									<Button
										class="k-color-brand-red"
										onClick={() =>
											rejectContactRequest(
												incomingContactRequest.contactRequestId,
											)}
									>
										{m.reject()}
									</Button>

									<Button
										onClick={() =>
											acceptContactRequest(
												incomingContactRequest.contactRequestId,
											)}
									>
										{m.accept()}
									</Button>
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
				<BlockTitle>{m.contacts()}</BlockTitle>
				<List strongIos insetIos>
					{#each contacts as [actorId, profile]}
						<ListItem
							link
							linkProps={{ href: `/direct-messages/${actorId}` }}
							title={profile.name}
						>
							{#snippet media()}
								<wa-avatar
									image={profile.avatar}
									initials={profile.name.slice(0, 2)}
								>
								</wa-avatar>
							{/snippet}
						</ListItem>
					{:else}
						<ListItem title={m.noContactsYet()} />
					{/each}
				</List>
			{/await}
		</div>
	</div>

</Page
>
