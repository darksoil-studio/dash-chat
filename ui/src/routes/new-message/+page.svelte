<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/avatar/avatar.js';
	import { m } from '$lib/paraglide/messages.js';
	import { mdiAccountMultiplePlus, mdiAccountPlus } from '@mdi/js';
	import type { ContactsStore } from 'dash-chat-stores';
	import { getContext } from 'svelte';
	import { useReactivePromise } from '../../stores/use-signal';
	import { wrapPathInSvg } from '../../utils/icon';
	import {
		Page,
		Navbar,
		NavbarBackLink,
		BlockTitle,
		List,
		ListItem,
		Button,
		Link,
		Preloader,
	} from 'konsta/svelte';

	const contactsStore: ContactsStore = getContext('contacts-store');

	const contacts = useReactivePromise(contactsStore.profilesForAllContacts);
</script>

<Page>
	<Navbar title={m.newMessage()} titleClass="opacity1" transparent={true}>
		{#snippet left()}
			<NavbarBackLink onClick={() => (window.location.href = '/')} />
		{/snippet}
	</Navbar>

	<div class="column" style="flex: 1">
		<div class="column center-in-desktop">
			<div class="column gap-4 m-4">
				<Link href="/add-contact">
					<Button tonal large class="w-full gap-2">
						<wa-icon src={wrapPathInSvg(mdiAccountPlus)}> </wa-icon>
						{m.addContact()}
					</Button>
				</Link>

				<Link href="/new-group">
					<Button tonal large class="w-full gap-2">
						<wa-icon src={wrapPathInSvg(mdiAccountMultiplePlus)}> </wa-icon>
						{m.newGroup()}
					</Button>
				</Link>
			</div>

			<BlockTitle>{m.contacts()}</BlockTitle>

			{#await $contacts}
				<div
					class="column"
					style="height: 100%; align-items: center; justify-content: center"
				>
					<Preloader />
				</div>
			{:then contacts}
				<List strong insetMaterial>
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
