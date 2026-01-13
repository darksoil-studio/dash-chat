<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/avatar/avatar.js';
	import type { ChatsStore } from 'dash-chat-stores';
	import { getContext } from 'svelte';
	import { useReactivePromise } from '$lib/stores/use-signal';
	import { mdiAccount } from '@mdi/js';
	import { wrapPathInSvg } from '$lib/utils/icon';
	import { m } from '$lib/paraglide/messages.js';
	import {
	Link,
		List,
		ListItem,
		Navbar,
		NavbarBackLink,
		Page,
		Preloader,
	} from 'konsta/svelte';
	import { page } from '$app/state';
	let chatId = page.params.chatId!;

	const chatsStore: ChatsStore = getContext('chats-store');
	const store = chatsStore.directMessagesChats(chatId);

	const peerProfile = useReactivePromise(store.peerProfile);
</script>

<Page>
	{#await $peerProfile}
		<div
			class="column"
			style="height: 100%; align-items: center; justify-content: center"
		>
			<Preloader />
		</div>
	{:then profile}
		<Navbar transparent={true}>
			{#snippet left()}
				<NavbarBackLink
					onClick={() => (window.location.href = `/direct-messages/${chatId}`)}
				/>
			{/snippet}

			{#snippet title()}
				<div
					class="gap-2 row"
					style="justify-content: start; align-items: center;"
				>
					<wa-avatar
						image={profile!.avatar}
						initials={profile!.name.slice(0, 2)}
						style="--size: 2.5rem"
					>
					</wa-avatar>
					<span>{profile!.name}</span>
				</div>
			{/snippet}
		</Navbar>
		<div class="column" style="flex: 1">
			<div class="column center-in-desktop">
				<div class="column m-10 gap-4" style="align-items: center">
					<wa-avatar
						image={profile?.avatar}
						initials={profile?.name.slice(0, 2)}
						style="--size: 64px;"
					>
					</wa-avatar>

					<span>{profile?.name} </span>
				</div>
			</div>
		</div>
	{/await}
</Page>
