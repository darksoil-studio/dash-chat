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
	} from 'konsta/svelte';

	const chatId = window.location.href.split('/').reverse()[0];

	const contactsStore: ContactsStore = getContext('contacts-store');
	const myActorId = useReactivePromise(contactsStore.myChatActorId);

	const chatsStore: ChatsStore = getContext('chats-store');
	const store = chatsStore.groupChats(chatId);

	const messages = useReactivePromise(store.messages);
	const info = useReactivePromise(store.info);
	const allMembers = useReactivePromise(store.allMembers);
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
		{#await $info then info}
			<Link
				href={`/group-chat/${chatId}/info`}
				class="gap-2"
				style="display: flex; justify-content: start; align-items: center; flex: 1"
			>
				<wa-avatar
					image={info.avatar}
					initials={info.name.slice(0, 2)}
					style="--size: 2.5rem"
				>
				</wa-avatar>
				<span>{info.name}</span>
			</Link>
		{/await}
	</Navbar>
	<div class="column">
		{#await $allMembers then members}
			<div class="center-in-desktop" style="flex:1">
				<div class="column m-2 gap-2">
					{#await $myActorId then myActorId}
						{#await $messages then messages}
							{#each messages as message}
								{#if myActorId == message.author}
									<Card
										raised
										style="align-self: end; background-color: var(--color-brand-primary); color: white; margin: 0"
									>
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
										<wa-avatar
											image={members[message.author].profile?.avatar}
											initials={members[message.author].profile?.name.slice(
												0,
												2,
											)}
											style="--size: 2.5rem"
										>
										</wa-avatar>
										<Card raised style="align-self: start; margin: 0">
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

			<div
				class="column pr-4 bottom-0 left-0 right-0 fixed"
				style="background-color: var(--background-color)"
			>
				<div class="row gap-1 center-in-desktop" style="align-items: center;">
					<List nested style="flex: 1">
						<ListInput
							type="textarea"
							outline
							bind:value={messageInput}
							inputStyle="min-height: 1em; padding: 0; max-height: 4em; resize: none"
							placeholder={m.typeMessage()}
						/>
					</List>

					<Button
						clear
						onClick={sendMessage}
						style="flex: 0; border-radius: 50%"
					>
						<wa-icon src={wrapPathInSvg(mdiSend)}> </wa-icon>
					</Button>
				</div>
			</div>
		{/await}
	</div>
</Page>
