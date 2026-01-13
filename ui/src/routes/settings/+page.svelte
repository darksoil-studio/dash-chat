<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/avatar/avatar.js';
	import type { ContactsStore } from 'dash-chat-stores';
	import { getContext } from 'svelte';
	import { useReactivePromise } from '$lib/stores/use-signal';
	import { mdiPencil } from '@mdi/js';
	import { wrapPathInSvg } from '$lib/utils/icon';
	import { m } from '$lib/paraglide/messages.js';
	import {
		Card,
		Link,
		List,
		ListItem,
		Navbar,
		NavbarBackLink,
		Page,
		Preloader,
		useTheme,
	} from 'konsta/svelte';

	const contactsStore: ContactsStore = getContext('contacts-store');

	const myProfile = useReactivePromise(contactsStore.myProfile);
	const theme = $derived(useTheme());
</script>

<Page>
	<Navbar title={m.settings()} titleClass="opacity1" transparent={true}>
		{#snippet left()}
			<NavbarBackLink onClick={() => (window.location.href = '/')} />
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
			<List class="center-in-desktop" strongIos nested={theme==='material'} inset>
				<ListItem
					link
					chevron={false}
					linkProps={{ href: '/settings/profile' }}
					title={myProfile?.name}
				>
					{#snippet media()}
						<wa-avatar
							image={myProfile?.avatar}
							initials={myProfile?.name.slice(0, 2)}
							style="--size: 64px"
						>
						</wa-avatar>
					{/snippet}
				</ListItem>
			</List>
		</div>
	{/await}
</Page>
