<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import { type ContactsStore } from 'dash-chat-stores';
	import { getContext } from 'svelte';
	import { useReactivePromise } from '$lib/stores/use-signal';
	import { wrapPathInSvg } from '$lib/utils/icon';
	import { mdiAccountGroup, mdiPencil, mdiSquareEditOutline } from '@mdi/js';
	import AllChats from '$lib/components/AllChats.svelte';
	import { Fab, Link, Navbar, Page, useTheme } from 'konsta/svelte';
	import { m } from '$lib/paraglide/messages';
	import { goto } from '$app/navigation';
	const theme = $derived(useTheme());

	const contactsStore: ContactsStore = getContext('contacts-store');
	const myProfile = useReactivePromise(contactsStore.myProfile);
</script>

<Page>
	<Navbar title={m.chats()} titleClass="opacity1" transparent={true}>
		{#snippet left()}
			{#await $myProfile then myProfile}
				<Link iconOnly href="/settings">
					<wa-avatar
						image={myProfile?.avatar}
						initials={myProfile?.name.slice(0, 2)}
						style="--size: 42px"
					>
					</wa-avatar>
				</Link>
			{/await}
		{/snippet}

		{#snippet right()}
			<Link iconOnly href="/contacts">
				<wa-icon src={wrapPathInSvg(mdiAccountGroup)}></wa-icon>
			</Link>

			{#if theme == 'ios'}
				<Link iconOnly href="/new-message">
					<wa-icon src={wrapPathInSvg(mdiSquareEditOutline)}> </wa-icon>
				</Link>
			{/if}
		{/snippet}
	</Navbar>

	<AllChats></AllChats>

	{#if theme == 'material'}
		<Fab
			class="fixed right-safe-4 bottom-safe-4 z-20"
			onClick={() => goto('/new-message')}
		>
			<wa-icon src={wrapPathInSvg(mdiPencil)}> </wa-icon>
		</Fab>
	{/if}
</Page>
