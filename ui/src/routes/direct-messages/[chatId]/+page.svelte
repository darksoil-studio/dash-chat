<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/avatar/avatar.js';
	import '@awesome.me/webawesome/dist/components/relative-time/relative-time.js';
	import '@awesome.me/webawesome/dist/components/format-date/format-date.js';

	import { useReactivePromise } from '../../../stores/use-signal';
	import { lessThanAMinuteAgo, moreThanAnHourAgo } from '../../../utils/time';
	import { getContext } from 'svelte';
	import type { ChatsStore, ContactsStore } from 'dash-chat-stores';
	import { wrapPathInSvg } from '@darksoil-studio/holochain-elements';
	import { mdiSend } from '@mdi/js';
	import { m } from '$lib/paraglide/messages.js';
	import {
		Page,
		Navbar,
		NavbarBackLink,
		Link,
		Button,
		Card,
		ListInput,
		List,
	} from 'konsta/svelte';

	const chatId = window.location.href.split('/').reverse()[0];

	const contactsStore: ContactsStore = getContext('contacts-store');
	const myActorId = useReactivePromise(contactsStore.myChatActorId);

	const chatsStore: ChatsStore = getContext('chats-store');
	const store = chatsStore.directMessagesChats(chatId);

	const messages = useReactivePromise(store.messages);
	const peerProfile = useReactivePromise(store.peerProfile);
	let messageInput = $state('');

	async function sendMessage() {
		const message = messageInput;
		if (!message || message.trim() === '') return;

		await store.sendMessage(message);
		messageInput = '';
	}
</script>

<Page>
	<Navbar>
		{#snippet left()}
			<NavbarBackLink onClick={() => (window.location.href = '/')} />
		{/snippet}
		{#snippet title()}
			{#await $peerProfile then profile}
				<Link
					href={`/direct-messages/${chatId}/info`}
					class="gap-2"
				style="display: flex; align-items: center;"
				>
					<wa-avatar
						image={profile!.avatar}
						initials={profile!.name.slice(0, 2)}
						style="--size: 2.5rem"
					>
					</wa-avatar>
					<span>{profile!.name}</span>
				</Link>
			{/await}
		{/snippet}
	</Navbar>

	<div
		class="column center-in-desktop"
		style="gap: 2rem; flex: 1; margin: 1rem; overflow-y: auto"
	>
		<div class="column" style="gap: 1rem; flex: 1">
			{#await $myActorId then myActorId}
				{#await $messages then messages}
					{#each messages as message}
						{#if myActorId == message.author}
							<Card
								class="max-w-[70%] self-end"
								style="padding: 0.5rem"
								colors={{ bgIos: 'bg-blue-100', bgMaterial: 'bg-blue-200' }}
							>
								<div class="row" style="gap: 0.5rem">
									<span>{message.content}</span>

									<div class="quiet text-xs">
										{#if lessThanAMinuteAgo(message.timestamp)}
											<span>{m.now()}</span>
										{:else if moreThanAnHourAgo(message.timestamp)}
											<wa-format-date
												hour="numeric"
												minute="numeric"
												hour-format="24"
												date={new Date(message.timestamp)}
											></wa-format-date>
										{:else}
											<wa-relative-time
												sync
												format="narrow"
												date={new Date(message.timestamp)}
											>
											</wa-relative-time>
										{/if}
									</div>
								</div>
							</Card>
						{:else}
							<Card class="max-w-[70%] self-start" style="padding: 0.5rem">
								<div class="row" style="gap: 0.5rem">
									<span>{message.content}</span>

									<div class="quiet text-xs">
										{#if lessThanAMinuteAgo(message.timestamp)}
											<span>{m.now()}</span>
										{:else if moreThanAnHourAgo(message.timestamp)}
											<wa-format-date
												hour="numeric"
												minute="numeric"
												hour-format="24"
												date={new Date(message.timestamp)}
											></wa-format-date>
										{:else}
											<wa-relative-time
												sync
												format="narrow"
												date={new Date(message.timestamp)}
											>
											</wa-relative-time>
										{/if}
									</div>
								</div>
							</Card>
						{/if}
					{/each}
				{/await}
			{/await}
		</div>

		<div class="row" style="align-items: center; gap: 0.5rem">
			<List nested>
				<ListInput
					type="textarea"
					bind:value={messageInput}
					inputStyle={{ minHeight: '40px', maxHeight: '120px' }}
					class="flex-1"
					placeholder={m.typeMessage?.() || 'Type a message...'}
				/>
			</List>

			<Button clear iconOnly onClick={sendMessage}>
				<wa-icon src={wrapPathInSvg(mdiSend)}> </wa-icon>
			</Button>
		</div>
	</div>
</Page>
