<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/avatar/avatar.js';
	import { useReactivePromise } from '$lib/stores/use-signal';
	import { getContext } from 'svelte';
	import { goto } from '$app/navigation';
	import type {
		ContactRequestId,
		ContactsStore,
		ContactRequest,
	} from 'dash-chat-stores';
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
		Toast,
	} from 'konsta/svelte';
	import { wrapPathInSvg } from '$lib/utils/icon';

	const contactsStore: ContactsStore = getContext('contacts-store');

	const contactRequests = useReactivePromise(contactsStore.contactRequests);
	const contacts = useReactivePromise(contactsStore.profilesForAllContacts);

	async function rejectContactRequest(contactRequest: ContactRequest) {
		try {
			// Actual rejection logic here
		} finally {
		}
	}

	let contactAddedToastName = $state<string | undefined>(undefined);
	let t: NodeJS.Timeout | undefined;
	async function acceptContactRequest(contactRequest: ContactRequest) {
		try {
			// Actual acceptance logic here
			await contactsStore.client.addContact(contactRequest.code);
			contactAddedToastName = contactRequest.profile.name;
			t = setTimeout(() => {
				if (t) clearTimeout(t);
				contactAddedToastName = undefined;
			}, 5000);
		} catch (e) {
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
			{#await $contactRequests}
				<div
					class="column"
					style="height: 100%; align-items: center; justify-content: center"
				>
					<Preloader />
				</div>
			{:then contactRequests}
				{#if contactRequests.length > 0}
					<BlockTitle>{m.contactRequests()}</BlockTitle>
					<List strongIos insetIos>
						{#each contactRequests as contactRequest}
							<ListItem title={contactRequest.profile.name}>
								{#snippet media()}
									<wa-avatar
										image={contactRequest.profile.avatar}
										initials={contactRequest.profile.name.slice(0, 2)}
									>
									</wa-avatar>
								{/snippet}
								{#snippet after()}
									<Button
										class="k-color-brand-red"
										onClick={() => rejectContactRequest(contactRequest)}
									>
										{m.reject()}
									</Button>

									<Button onClick={() => acceptContactRequest(contactRequest)}>
										{m.accept()}
									</Button>
								{/snippet}
							</ListItem>
						{/each}
					</List>
				{/if}
			{/await}

			{#if contactAddedToastName !== undefined}
				<Toast position="center" opened={true}
					>{m.contactAdded({
						name: contactAddedToastName!,
					})}
				</Toast>
			{/if}

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
</Page>
