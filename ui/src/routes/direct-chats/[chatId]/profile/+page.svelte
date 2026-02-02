<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/avatar/avatar.js';
	import { fullName, type ChatsStore } from 'dash-chat-stores';
	import { getContext } from 'svelte';
	import { goto } from '$app/navigation';
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
	const store = chatsStore.directChats(chatId);

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
				<NavbarBackLink onClick={() => goto(`/direct-chats/${chatId}`)} />
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
					<span>{fullName(profile!)}</span>
				</div>
			{/snippet}
		</Navbar>
		<div class="column" style="flex: 1">
			<div class="column center-in-desktop">
				<div class="column m-6 gap-2" style="align-items: center">
					<wa-avatar
						image={profile?.avatar}
						initials={profile?.name.slice(0, 2)}
						style="--size: 80px;"
					>
					</wa-avatar>

					<span class="text-lg font-semibold">{fullName(profile!)} </span>
				</div>
			</div>
		</div>
	{/await}
</Page>
