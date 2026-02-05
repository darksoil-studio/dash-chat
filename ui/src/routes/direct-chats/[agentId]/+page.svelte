<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/avatar/avatar.js';
	import '@awesome.me/webawesome/dist/components/relative-time/relative-time.js';
	import '@awesome.me/webawesome/dist/components/format-date/format-date.js';
	import { m, yesterday } from '$lib/paraglide/messages.js';

	import { useReactivePromise } from '$lib/stores/use-signal';
	import {
		beforeYesterday,
		inYesterday,
		lessThanAMinuteAgo,
		moreThanAnHourAgo,
		moreThanAWeekAgo,
		moreThanAYearAgo,
	} from '$lib/utils/time';
	import { getContext, onMount, tick } from 'svelte';
	import { goto } from '$app/navigation';
	import {
		fullName,
		toPromise,
		type ChatsStore,
		type ContactCode,
		type ContactRequest,
		type ContactsStore,
		type DeviceId,
		type Hash,
		type Message,
	} from 'dash-chat-stores';
	import type { AddContactError } from 'dash-chat-stores';
	import { wrapPathInSvg } from '$lib/utils/icon';
	import {
		mdiSend,
		mdiAlert,
		mdiAccountQuestion,
		mdiAccountGroup,
	} from '@mdi/js';
	import {
		Page,
		Navbar,
		NavbarBackLink,
		Link,
		Button,
		Card,
		useTheme,
	} from 'konsta/svelte';
	import { page } from '$app/state';
	import { showToast } from '$lib/utils/toasts';
	import type { Action } from 'svelte/action';
	import MessageInput from '$lib/components/MessageInput.svelte';
	let agentId = page.params.agentId!;

	const contactsStore: ContactsStore = getContext('contacts-store');
	const myAgentId = useReactivePromise(contactsStore.myAgentId);
	const myDeviceId = useReactivePromise(contactsStore.myDeviceId);

	const chatsStore: ChatsStore = getContext('chats-store');
	const store = chatsStore.directChats(agentId);

	const messagesSets = useReactivePromise(store.messageSets);
	const peerProfile = useReactivePromise(store.peerProfile);
	const contactRequest = useReactivePromise(store.contactRequest);

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
			// Wait for the new operation to redirect to the home page
			// This prevents the rejected contact request to render on the home page before it's
			setTimeout(() => {
				showToast(m.contactRequestRejected());

				goto('/');
			});
		} catch (e) {
			console.error(e);
			showToast(m.errorUnexpected(), 'error');
		}
	}

	let messageText = $state('');
	let showEmojiPicker = $state(false);
	let messageInputHeight: string = $state('');

	const scrollIsAtBottom = () => {
		const pageEl = document.querySelector('.messages-page') as HTMLDivElement;
		if (!pageEl) return;
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
	store.onNewMessage(async () => {
		if (scrollIsAtBottom()) {
			// Wait for the message to get rendered in the UI
			setTimeout(() => {
				scrollToBottom();
			});
		}
	});

	const messageClass = (messageSetLength: number, index: number) => {
		if (messageSetLength <= 1) return '';
		if (index === 0) return 'first-message';
		if (index === messageSetLength - 1) return 'last-message';
		return 'middle-message';
	};

	// Track visible messages to mark as read
	let observer: IntersectionObserver | undefined;
	const visibleMessages: Set<Hash> = new Set();
	let markReadTimeout: ReturnType<typeof setTimeout>;

	onMount(() => {
		observer = new IntersectionObserver(
			entries => {
				for (const entry of entries) {
					const hash = entry.target.getAttribute('data-message-hash');
					if (hash && entry.isIntersecting) {
						visibleMessages.add(hash);
					}
				}
				// Debounce the mark-as-read call
				clearTimeout(markReadTimeout);
				markReadTimeout = setTimeout(() => {
					if (visibleMessages.size > 0) {
						store.markAsRead(Array.from(visibleMessages));
						visibleMessages.clear();
					}
				}, 500);
			},
			{ threshold: 0.5 },
		);

		return () => {
			observer?.disconnect();
			clearTimeout(markReadTimeout);
		};
	});

	// Svelte action to observe message elements for read tracking
	const observeMessage: Action<HTMLElement, Hash> = (node, hash) => {
		node.setAttribute('data-message-hash', hash);
		observer?.observe(node);
		return {
			destroy() {
				observer?.unobserve(node);
			},
		};
	};
	const theme = $derived(useTheme());
</script>

<Page class="messages-page">
	{#await $peerProfile then profile}
		{#await $contactRequest then contactRequest}
			<Navbar
				transparent={true}
				titleClass="opacity1 w-full"
				centerTitle={false}
			>
				{#snippet left()}
					<NavbarBackLink onClick={() => goto('/')} />
				{/snippet}
				{#snippet title()}
					{#if profile}
						<Link
							class="gap-2"
							style="display: flex; justify-content: start; align-items: center;"
							href={`/direct-chats/${agentId}/profile`}
						>
							<wa-avatar
								image={profile!.avatar}
								initials={profile!.name.slice(0, 2)}
								style="--size: 2.5rem"
							>
							</wa-avatar>
							<span>{fullName(profile!)}</span>
						</Link>
					{/if}
				{/snippet}
			</Navbar>

			<div class="column">
				{#await $myDeviceId then myDeviceId}
					{#await $messagesSets then messagesSetsInDays}
						<div
							use:scrolltobottom
							class="center-in-desktop column"
							style={`padding-bottom: ${messageInputHeight}`}
						>
							{#if profile}
								<div class="column" style="align-items: center">
									<Link
										class="column my-6 gap-2"
										href={`/direct-chats/${agentId}/profile`}
									>
										<wa-avatar
											image={profile.avatar}
											initials={profile.name.slice(0, 2)}
											style="--size: 80px;"
										>
										</wa-avatar>
										<span class="text-lg font-semibold"
											>{fullName(profile)}</span
										>
									</Link>
								</div>
							{/if}
							<div class="row justify-center">
								<Card
									raised
									class="no-padding rounded-xl"
									style="background-color: transparent"
								>
									<div class="flex flex-col items-center gap-3 p-3 text-center">
										{#if contactRequest}
											<div class="flex items-center gap-2 text-amber-600">
												<wa-icon
													class="small-icon"
													src={wrapPathInSvg(mdiAlert)}
												></wa-icon>
												<span class="font-semibold">{m.reviewCarefully()}</span>
											</div>
										{/if}
										<div
											class="flex flex-col gap-2 text-sm text-gray-700 dark:text-gray-300"
										>
											<div class="flex items-center justify-center gap-2">
												<wa-icon
													class="small-icon"
													src={wrapPathInSvg(mdiAccountQuestion)}
												></wa-icon>
												<span>{m.profileNamesNotVerified()}</span>
											</div>
											<div class="flex items-center justify-center gap-2">
												<wa-icon
													class="small-icon"
													src={wrapPathInSvg(mdiAccountGroup)}
												></wa-icon>
												<span>{m.noGroupsInCommon()}</span>
											</div>
										</div>
										{#if contactRequest}
											<div class="row justify-center">
												<Button rounded tonal small>
													{m.securityTips()}
												</Button>
											</div>
										{/if}
									</div>
								</Card>
							</div>

							<div class="column m-2 gap-1">
								{#each messagesSetsInDays as messageSetInDay}
									<div class="sticky-day-tag quiet">
										{#if moreThanAYearAgo(messageSetInDay.day.valueOf())}
											<wa-format-date
												month="numeric"
												year="numeric"
												day="numeric"
												date={messageSetInDay.day}
											></wa-format-date>
										{:else if beforeYesterday(messageSetInDay.day.valueOf())}
											<wa-format-date
												month="short"
												day="numeric"
												weekday="narrow"
												date={messageSetInDay.day}
											></wa-format-date>
										{:else if inYesterday(messageSetInDay.day.valueOf())}
											{m.yesterday()}
										{:else}
											{m.today()}
										{/if}
									</div>

									{#each messageSetInDay.eventsSets as messageSet}
										<div class="column" style="gap: 1px">
											{#each messageSet as [hash, message], i}
												{#if myDeviceId == message.author}
													<Card
														raised
														class={`${messageClass(messageSet.length, i)} message my-message`}
													>
														<div
															class="row gap-2 mx-1"
															style="align-items: center"
														>
															<span>{message.content}</span>

															{#if i === messageSet.length - 1}
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
															{/if}
														</div>
													</Card>
												{:else}
													<div class="row gap-2 m-0" use:observeMessage={hash}>
														<Card
															raised
															class={`${messageClass(messageSet.length, i)} message others-message`}
														>
															<div
																class="row gap-2 mx-1"
																style="align-items: center"
															>
																<span>{message.content}</span>

																{#if i === messageSet.length - 1}
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
																{/if}
															</div>
														</Card>
													</div>
												{/if}
											{/each}
										</div>
									{/each}
								{/each}
							</div>
						</div>
					{/await}
				{/await}

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
			</div>
		{/await}
	{/await}
</Page>
