<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/avatar/avatar.js';
	import type { ContactsStore } from 'dash-chat-stores';
	import { getContext } from 'svelte';
	import { goto } from '$app/navigation';
	import { useReactivePromise } from '$lib/stores/use-signal';
	import { mdiAccount, mdiPencil } from '@mdi/js';
	import { wrapPathInSvg } from '$lib/utils/icon';
	import { m } from '$lib/paraglide/messages.js';
	import { fullName } from 'dash-chat-stores'
	import {
		Button,
		Card,
		Link,
		List,
		ListItem,
		Navbar,
		NavbarBackLink,
		Page,
		Preloader,
	} from 'konsta/svelte';

	const contactsStore: ContactsStore = getContext('contacts-store');

	const myProfile = useReactivePromise(contactsStore.myProfile);
</script>

<Page>
	<Navbar title={m.profile()} titleClass="opacity1" transparent={true}>
		{#snippet left()}
			<NavbarBackLink onClick={() => goto('/settings')} />
		{/snippet}
	</Navbar>

	{#await $myProfile}
		<div
			class="column"
			style="height: 100%; align-items: center; justify-content: center"
		>
			<Preloader />
		</div>
	{:then myProfile}
		<div class="column" style="flex: 1">
			<div class="column center-in-desktop">
				<div class="column m-10 gap-4" style="align-items: center">
					<wa-avatar
						image={myProfile?.avatar}
						initials={myProfile?.name.slice(0, 2)}
						style="--size: 80px;"
					>
					</wa-avatar>

					<Button tonal style="width: auto" rounded onClick={()=>goto('/settings/profile/edit-photo')}>{m.editPhoto()}</Button>
				</div>

				<List nested strongIos insetIos>
					<ListItem
						title={fullName(myProfile!)}
						link
						linkProps={{ href: '/settings/profile/edit-name' }}
					>
						{#snippet media()}
							<wa-icon src={wrapPathInSvg(mdiAccount)}></wa-icon>
						{/snippet}
					</ListItem>
				</List>
			</div>
		</div>
	{/await}
</Page>
