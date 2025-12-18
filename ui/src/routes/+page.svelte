<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/badge/badge.js';
	import '@awesome.me/webawesome/dist/components/button/button.js';
	import { type ContactsStore } from 'dash-chat-stores';
	import { getContext } from 'svelte';
	import { useReactivePromise } from '../stores/use-signal';
	import Avatar from '../components/Avatar.svelte';
	import { wrapPathInSvg } from '@darksoil-studio/holochain-elements';
	import { mdiAccountGroup, mdiSquareEditOutline } from '@mdi/js';
	import AllChats from '../components/AllChats.svelte';
	import { Badge, Link, Navbar, Page, useTheme } from 'konsta/svelte';
	import { m } from '$lib/paraglide/messages';
	const theme = $derived(useTheme());

	const contactsStore: ContactsStore = getContext('contacts-store');
	const myProfile = useReactivePromise(contactsStore.myProfile);
	const incomingContactRequests = useReactivePromise(
		contactsStore.incomingContactRequests,
	);
</script>

<Page>
	<Navbar title={m.chats()} class="top-0 sticky">
		{#snippet left()}
			{#await $myProfile then myProfile}
				<Link iconOnly href="/my-profile">
					<wa-avatar
						image={myProfile?.avatar}
						initials={myProfile?.name.slice(0, 2)}
						style="--size:42px"
					>
					</wa-avatar>
				</Link>
			{/await}
		{/snippet}

		<div
			class="row"
			style="gap: var(--wa-space-s); align-items: center; justify-content: end; flex: 1;"
		>
			<Link
				href="/contacts"
				iconOnly
				style="position: relative; padding-right: 0; padding-left: 0"
			>
				<wa-icon src={wrapPathInSvg(mdiAccountGroup)}> </wa-icon>
				{#await $incomingContactRequests then incomingContactRequests}
					{#if incomingContactRequests.length > 0}
						<Badge style="position: absolute; right: -4px; bottom: -4px">
							{incomingContactRequests.length}
						</Badge>
					{/if}
				{/await}
			</Link>

			<Link
				iconOnly
				href="/new-message"
				style="padding-right: 0; padding-left: 0"
			>
				<wa-icon src={wrapPathInSvg(mdiSquareEditOutline)}> </wa-icon>
			</Link>
			{#if theme == 'material'}
				<div></div>
			{/if}
		</div>
	</Navbar>

	<AllChats></AllChats>
</Page>
