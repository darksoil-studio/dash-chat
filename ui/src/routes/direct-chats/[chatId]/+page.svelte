<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/avatar/avatar.js';
	import '@awesome.me/webawesome/dist/components/relative-time/relative-time.js';
	import '@awesome.me/webawesome/dist/components/format-date/format-date.js';
	import { m } from '$lib/paraglide/messages.js';

	import { useReactivePromise } from '$lib/stores/use-signal';
	import { lessThanAMinuteAgo, moreThanAnHourAgo } from '$lib/utils/time';
	import { getContext, onMount, tick } from 'svelte';
	import { goto } from '$app/navigation';
	import type {
		ChatsStore,
		ContactCode,
		ContactRequest,
		ContactsStore,
		DeviceId,
		Message,
	} from 'dash-chat-stores';
	import type { AddContactError } from 'dash-chat-stores';
	import { wrapPathInSvg } from '$lib/utils/icon';
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
	import { page } from '$app/state';
	import { showToast } from '$lib/utils/toasts';
	import { get } from 'svelte/store';
	import { watcher } from 'signalium';
	import type { Action } from 'svelte/action';
	import MessageInput from '$lib/components/MessageInput.svelte';
	let chatId = page.params.chatId!;

	const contactsStore: ContactsStore = getContext('contacts-store');
	const myAgentId = useReactivePromise(contactsStore.myAgentId);
	const myDeviceId = useReactivePromise(contactsStore.myDeviceId);

	const chatsStore: ChatsStore = getContext('chats-store');
	const store = chatsStore.directChats(chatId);

	const messages = useReactivePromise(store.messages);
	const peerProfile = useReactivePromise(store.peerProfile);
	const contactRequest = useReactivePromise(store.getContactRequest);

	async function acceptContactRequest(contactRequest: ContactRequest) {
		try {
			await contactsStore.client.addContact(contactRequest.code);
			showToast(m.contactAccepted());
		} catch (e) {
			console.error(e);
			const error = e as AddContactError;
			switch (error.kind) {
				case 'ProfileNotCreated':
					showToast(m.errorAddContactProfileRequired(), 'error');
					break;
				case 'InitializeTopic':
				case 'AuthorOperation':
				case 'CreateQrCode':
				case 'CreateDirectChat':
					showToast(m.errorAddContact(), 'error');
					break;
				default:
					showToast(m.errorUnexpected(), 'error');
			}
		}
	}

	async function rejectContactRequest(contactRequest: ContactRequest) {
		try {
			await contactsStore.client.rejectContactRequest(
				contactRequest.code.agent_id,
			);
			showToast(m.contactRequestRejected());

			goto('/');
		} catch (e) {
			console.error(e);
			showToast(m.errorUnexpected(), 'error');
		}
	}

	let messageText = $state('');
	let showEmojiPicker = $state(false);
	let messageInputHeight: string = $state('');

	const scrollIsAtBottom = () => {
		const pageEl = document.querySelector('.messages-page')! as HTMLDivElement;
		return pageEl.scrollTop === pageEl.scrollHeight - pageEl.offsetHeight;
	};

	const scrollToBottom = (animate = true) => {
		const pageEl = document.querySelector('.messages-page')! as HTMLDivElement;
		pageEl.scrollTo({
			top: pageEl.scrollHeight - pageEl.offsetHeight,
			behavior: animate ? 'smooth' : 'auto',
		});
	};

	const scrolltobottom: Action = () => {
		tick().then(() => {
			scrollToBottom(false);
		});
	};

	async function sendMessage() {
		const message = messageText;

		if (!message || message.trim() === '') return;

		await store.sendMessage(message);
		messageText = '';
		// Wait for the message to get rendered in the UI
		setTimeout(() => {
			scrollToBottom();
		});
	}
	const theme = $derived(useTheme());
</script>

{#await $peerProfile then profile}
	<Page class="messages-page">
		<Navbar transparent={true} titleClass="opacity1 w-full" centerTitle={false}>
			{#snippet left()}
				<NavbarBackLink onClick={() => goto('/')} />
			{/snippet}
			{#snippet title()}
				{#if profile}
					<Link
						class="gap-2"
						style="display: flex; justify-content: start; align-items: center;"
						href={`/direct-chats/${chatId}/profile`}
					>
						<wa-avatar
							image={profile!.avatar}
							initials={profile!.name.slice(0, 2)}
							style="--size: 2.5rem"
						>
						</wa-avatar>
						<span>{profile!.name}</span>
					</Link>
				{/if}
			{/snippet}
		</Navbar>

				<div class="column" >
		{#await $myDeviceId then myDeviceId}
			{#await $messages then messages}
					<div
use:scrolltobottom
						class="center-in-desktop column"
						style={`padding-bottom: ${messageInputHeight}`}
					>
						{#if profile}
							<div class="column" style="align-items: center">
								<Link
									class="column my-6 gap-2"
									href={`/direct-chats/${chatId}/profile`}
								>
									<wa-avatar
										image={profile.avatar}
										initials={profile.name.slice(0, 2)}
										style="--size: 80px;"
									>
									</wa-avatar>
									<span class="text-lg font-semibold">{profile.name}</span>
								</Link>
							</div>
						{/if}

						<div class="column m-2 gap-2">
							{#each messages as message}
								{#if myDeviceId == message.author}
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
						</div>
					</div>
			{/await}
		{/await}

					{#await $contactRequest then contactRequest}
						{#if contactRequest}
							<Card class="center-in-desktop p-1 fixed bottom-1">
								<div class="column gap-2 items-center justify-center">
									<span style="flex: 1"
										>{m.contactRequestBanner({
											name: contactRequest.profile.name,
										})}</span
									>
									<div class="flex gap-2">
										<Button
											class="k-color-brand-red"
											rounded
											tonal
											onClick={() => rejectContactRequest(contactRequest)}
											>{m.reject()}</Button
										>
										<Button
											tonal
											rounded
											onClick={() => acceptContactRequest(contactRequest)}
											>{m.accept()}</Button
										>
									</div>
								</div>
							</Card>
						{:else}
							<MessageInput
								bind:value={messageText}
								bind:height={messageInputHeight}
								onSend={sendMessage}
								onInput={async () => {
									if (scrollIsAtBottom()) {
										await tick();
										scrollToBottom();
									}
								}}
								onEmojiClick={() => (showEmojiPicker = true)}
							/>
						{/if}
					{/await}
				</div>
	</Page>
{/await}
