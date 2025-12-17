<script lang="ts">
	import '@awesome.me/webawesome/dist/components/textarea/textarea.js';
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/button/button.js';
	import '@awesome.me/webawesome/dist/components/relative-time/relative-time.js';
	import '@awesome.me/webawesome/dist/components/format-date/format-date.js';
	import type WaTextarea from '@awesome.me/webawesome/dist/components/textarea/textarea.js';

	import { useReactivePromise } from '../../../stores/use-signal';
	import { lessThanAMinuteAgo, moreThanAnHourAgo } from '../../../utils/time';
	import { getContext } from 'svelte';
	import type { ChatsStore, ContactsStore } from 'dash-chat-stores';
	import { wrapPathInSvg } from '@darksoil-studio/holochain-elements';
	import { mdiArrowLeft, mdiSend } from '@mdi/js';

	const chatId = window.location.href.split('/').reverse()[0];

	const contactsStore: ContactsStore = getContext('contacts-store');
	const myActorId = useReactivePromise(contactsStore.myChatActorId);

	const chatsStore: ChatsStore = getContext('chats-store');
	const store = chatsStore.directMessagesChats(chatId);

	const messages = useReactivePromise(store.messages);
	const peerProfile = useReactivePromise(store.peerProfile);
	let textarea: WaTextarea;

	async function sendMessage() {
		const message = textarea.value;
		if (!message || message.trim() === '') return;

		await store.sendMessage(message);
		textarea.value = '';
	}
</script>

<div class="top-bar" style="gap: 0">
	<wa-button class="circle" appearance="plain" href="/">
		<wa-icon src={wrapPathInSvg(mdiArrowLeft)}> </wa-icon>
	</wa-button>

	<wa-button
		style="flex: 1"
		href={`/direct-messages/${chatId}/info`}
		appearance="plain"
	>
		{#await $peerProfile then profile}
			<wa-avatar
				slot="start"
				image={profile!.avatar}
				initials={profile!.name.slice(0, 2)}
			>
			</wa-avatar>
			<span>{profile!.name}</span>
		{/await}
	</wa-button>
</div>

<div
	class="column center-in-desktop"
	style="gap: var(--wa-space-l); flex: 1;  margin: var(--wa-space-m)"
>
	<div class="column" style="gap: var(--wa-space-m); flex: 1;">
		{#await $myActorId then myActorId}
			{#await $messages then messages}
				{#each messages as message}
					{#if myActorId == message.author}
						<wa-card class="wa-dark" style="align-self: end">
							<div
								class="row"
								style="gap: var(--wa-space-s);"
							>
								<span>{message.content}</span>

								<div class="quiet">
									{#if lessThanAMinuteAgo(message.timestamp)}
										<span>now</span>
									{:else if moreThanAnHourAgo(message.timestamp)}
										<wa-format-date
											hour="numeric"
											minute="numeric"
											hour-format="24"
											date={new Date(message.timestamp)}
										></wa-format-date>
									{:else}
										<wa-relative-time
											style=""
											sync
											format="narrow"
											date={new Date(message.timestamp)}
										>
										</wa-relative-time>
									{/if}
								</div>
							</div></wa-card
						>
					{:else}
						<wa-card style="align-self: start">
							<div class="row" style="gap: var(--wa-space-s)">
								<span>{message.content}</span>

								<div class="quiet">
									{#if lessThanAMinuteAgo(message.timestamp)}
										<span>now</span>
									{:else if moreThanAnHourAgo(message.timestamp)}
										<wa-format-date
											hour="numeric"
											minute="numeric"
											hour-format="24"
											date={new Date(message.timestamp)}
										></wa-format-date>
									{:else}
										<wa-relative-time
											style=""
											sync
											format="narrow"
											date={new Date(message.timestamp)}
										>
										</wa-relative-time>
									{/if}
								</div>
							</div></wa-card
						>
					{/if}
				{/each}
			{/await}
		{/await}
	</div>

	<div class="row" style="align-items: center; gap: var(--wa-space-s)">
		<wa-textarea resize="auto" rows="1" bind:this={textarea} style="flex: 1">
		</wa-textarea>

		<wa-button class="circle" appearance="plain" onclick={sendMessage}>
			<wa-icon src={wrapPathInSvg(mdiSend)}> </wa-icon>
		</wa-button>
	</div>
</div>

<style>
	wa-avatar {
		--size: 2.5rem;
	}

	wa-button::part(label) {
		flex: 1;
	}

	wa-card::part(body) {
		padding: 8px;
	}
</style>
