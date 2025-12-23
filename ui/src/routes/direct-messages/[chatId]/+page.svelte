<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/avatar/avatar.js';
	import '@awesome.me/webawesome/dist/components/relative-time/relative-time.js';
	import '@awesome.me/webawesome/dist/components/format-date/format-date.js';
	import { m } from '$lib/paraglide/messages.js';

	import { useReactivePromise } from '../../../stores/use-signal';
	import { lessThanAMinuteAgo, moreThanAnHourAgo } from '../../../utils/time';
	import { getContext } from 'svelte';
	import type { ChatsStore, ContactsStore } from 'dash-chat-stores';
	import { wrapPathInSvg } from '@darksoil-studio/holochain-elements';
	import { mdiSend } from '@mdi/js';
	import {
		Page,
		Navbar,
		NavbarBackLink,
		Link,
		Button,
		Card,
		ListInput,
		List,
		Messagebar,
		ToolbarPane,
		Icon,
		useTheme,
	} from 'konsta/svelte';

	const chatId = window.location.href.split('/').reverse()[0];

	const contactsStore: ContactsStore = getContext('contacts-store');
	const myActorId = useReactivePromise(contactsStore.myChatActorId);

	const chatsStore: ChatsStore = getContext('chats-store');
	const store = chatsStore.directMessagesChats(chatId);

	const messages = useReactivePromise(store.messages);
	const peerProfile = useReactivePromise(store.peerProfile);
	let messageText = $state('');
	let isClickable = $state(false);
	let inputOpacity = $state(0.3);
	const onMessageTextChange = (e: InputEvent) => {
		messageText = (e.target as HTMLInputElement).value;
		isClickable = messageText.trim().length > 0;
		inputOpacity = messageText ? 1 : 0.3;
	};

	async function sendMessage() {
		const message = messageText;
		if (!message || message.trim() === '') return;

		await store.sendMessage(message);
		messageText = '';
	}
	const theme = $derived(useTheme());
</script>

<Page style={theme === 'material' ? 'height: calc(100vh - 57px)' : ''}>
	<Navbar transparent={true}>
		{#snippet left()}
			<NavbarBackLink onClick={() => (window.location.href = '/')} />
		{/snippet}
		{#await $peerProfile then profile}
			<div
				class="gap-2"
				style="display: flex; justify-content: start; align-items: center; flex: 1"
			>
				<wa-avatar
					image={profile!.avatar}
					initials={profile!.name.slice(0, 2)}
					style="--size: 2.5rem"
				>
				</wa-avatar>
				<span>{profile!.name}</span>
			</div>
		{/await}
	</Navbar>

	<div class="column">
		<div class="center-in-desktop" style="flex:1">
			<div class="column m-2 gap-2">
				{#await $myActorId then myActorId}
					{#await $messages then messages}
						{#each messages as message}
							{#if myActorId == message.author}
								<Card raised class="message my-message">
									<div class="row gap-2" style="align-items: center">
										<span>{message.content}</span>

										<div class="dark-quiet text-xs">
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
								<div class="row gap-2 m-0">
									<Card raised class="message others-message">
										<div class="row gap-2" style="align-items: center">
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
								</div>
							{/if}
						{/each}
					{/await}
				{/await}
			</div>
		</div>

		<Messagebar
			placeholder={m.typeMessage()}
			onInput={onMessageTextChange}
			value={messageText}
		>
			{#snippet right()}
				<ToolbarPane class="ios:h-10">
					<Link
						iconOnly
						onClick={() => (isClickable ? sendMessage() : undefined)}
						style="opacity: {inputOpacity}; cursor: {isClickable
							? 'pointer'
							: 'default'}"
					>
						<Icon>
							<wa-icon src={wrapPathInSvg(mdiSend)}> </wa-icon>
						</Icon>
					</Link>
				</ToolbarPane>
			{/snippet}
		</Messagebar>
	</div>
</Page>
