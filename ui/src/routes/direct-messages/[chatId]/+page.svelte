<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/avatar/avatar.js';
	import '@awesome.me/webawesome/dist/components/relative-time/relative-time.js';
	import '@awesome.me/webawesome/dist/components/format-date/format-date.js';
	import { m } from '$lib/paraglide/messages.js';

	import { useReactivePromise } from '$lib/stores/use-signal';
	import {
		lessThanAMinuteAgo,
		moreThanAnHourAgo,
		sleep,
	} from '$lib/utils/time';
	import { getContext } from 'svelte';
	import { goto } from '$app/navigation';
	import type {
		ChatsStore,
		ContactCode,
		ContactRequest,
		ContactsStore,
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
		Toast,
	} from 'konsta/svelte';
	import { page } from '$app/state';
	import { TOAST_TTL_MS } from '$lib/utils/toasts';
	let chatId = page.params.chatId!;

	const contactsStore: ContactsStore = getContext('contacts-store');
	const myActorId = useReactivePromise(contactsStore.myAgentId);

	const chatsStore: ChatsStore = getContext('chats-store');
	const store = chatsStore.directMessagesChats(chatId);

	const messages = useReactivePromise(store.messages);
	const peerProfile = useReactivePromise(store.peerProfile);
	const contactRequest = useReactivePromise(store.getContactRequest);

	let acceptedToastOpen = $state(false);
	let errorMessage = $state<string | undefined>(undefined);

	async function acceptContactRequest(contactRequest: ContactRequest) {
		try {
			await contactsStore.client.addContact(contactRequest.code);
			acceptedToastOpen = true;
			setTimeout(() => {
				acceptedToastOpen = false;
			}, TOAST_TTL_MS);
		} catch (e) {
			const error = e as AddContactError;
			switch (error.kind) {
				case 'ProfileNotCreated':
					errorMessage = m.errorAddContactProfileRequired();
					break;
				case 'InitializeTopic':
				case 'AuthorOperation':
				case 'CreateQrCode':
				case 'CreateDirectChat':
					errorMessage = m.errorAddContact();
					break;
				default:
					errorMessage = m.errorUnexpected();
			}
			setTimeout(() => {
				errorMessage = undefined;
			}, TOAST_TTL_MS);
		}
	}

	async function rejectContactRequest(contactRequest: ContactRequest) {
		try {
			await contactsStore.client.rejectContactRequest(
				contactRequest.code.agent_id,
			);
			// Without set timeout, the route is navigated to before the notification is
			// processed, showing the contact request still active
			setTimeout(()=>goto('/'))
		} catch (e) {
			errorMessage = m.errorUnexpected();
			setTimeout(() => {
				errorMessage = undefined;
			}, TOAST_TTL_MS);
		}
	}

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

<Page>
	<Navbar transparent={true} titleClass="opacity1 w-full" centerTitle={false}>
		{#snippet left()}
			<NavbarBackLink onClick={() => goto('/')} />
		{/snippet}
		{#snippet title()}
			{#await $peerProfile then profile}
				{#if profile}
					<Link
						class="gap-2"
						style="display: flex; justify-content: start; align-items: center;"
						href={`/direct-messages/${chatId}/profile`}
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
			{/await}
		{/snippet}
	</Navbar>

	<div class={`column ${theme === 'ios' ? 'pb-16' : ''}`}>
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

		{#await $contactRequest then contactRequest}
			{#if contactRequest}
				<Card class="center-in-desktop p-2 fixed bottom-2">
					<div class="flex flex-row items-center justify-center">
						<span style="flex: 1"
							>{m.contactRequestBanner({
								name: contactRequest.profile.name,
							})}</span
						>
						<div class="flex gap-2">
							<Button
								rounded
								clear
								onClick={() => rejectContactRequest(contactRequest)}
								>{m.reject()}</Button
							>
							<Button
								rounded
								onClick={() => acceptContactRequest(contactRequest)}
								>{m.accept()}</Button
							>
						</div>
					</div>
				</Card>
			{:else}
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
			{/if}
		{/await}
	</div>

	<Toast position="center" opened={acceptedToastOpen}>
		{m.contactAccepted()}
	</Toast>
	<Toast
		position="center"
		class="k-color-brand-red"
		opened={errorMessage !== undefined}>{errorMessage}</Toast
	>
</Page>
