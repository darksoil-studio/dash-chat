<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/badge/badge.js';
	import '@awesome.me/webawesome/dist/components/button/button.js';
	import { ChatsStore, type ContactsStore } from 'dash-chat-stores';
	import { getContext } from 'svelte';
	import { useReactivePromise } from '../stores/use-signal';
	import Avatar from '../components/Avatar.svelte';
	import { wrapPathInSvg } from '@darksoil-studio/holochain-elements';
	import { mdiAccountGroup, mdiSquareEditOutline } from '@mdi/js';
	import AllChats from '../components/AllChats.svelte';

	const contactsStore: ContactsStore = getContext('contacts-store');
	const myChatActorId = useReactivePromise(contactsStore.myChatActorId);
	const incomingContactRequests = useReactivePromise(
		contactsStore.incomingContactRequests,
	);
</script>

{#await $myChatActorId then myChatActorId}
	<div class="column">
		<div class="top-bar">
			<a href="/my-profile">
				<Avatar chatActorId={myChatActorId}></Avatar>
			</a>

			<div style="flex: 1"></div>

			<wa-button
				class="circle"
				href="/contacts"
				appearance="plain"
				style="position: relative"
			>
				<wa-icon src={wrapPathInSvg(mdiAccountGroup)}> </wa-icon>
				{#await $incomingContactRequests then incomingContactRequests}
					{#if incomingContactRequests.length > 0}
						<wa-badge
							variant="brand"
							pill
							style="position:absolute; right: -4px; bottom: -2px"
							>{incomingContactRequests.length}
						</wa-badge>
					{/if}
				{/await}
			</wa-button>

			<wa-button class="circle" href="/new-message" appearance="plain">
				<wa-icon src={wrapPathInSvg(mdiSquareEditOutline)}> </wa-icon>
			</wa-button>
		</div>
	</div>
{/await}

<AllChats></AllChats>
