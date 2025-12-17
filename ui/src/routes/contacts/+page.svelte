<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/button/button.js';
	import '@awesome.me/webawesome/dist/components/avatar/avatar.js';
	import { useReactivePromise } from '../../stores/use-signal';
	import { getContext } from 'svelte';
	import type { ContactRequestId, ContactsStore } from 'dash-chat-stores';
	import Avatar from '../../components/Avatar.svelte';
	import { wrapPathInSvg } from '@darksoil-studio/holochain-elements';
	import { mdiAccountPlus } from '@mdi/js';
	import WaButton from '@awesome.me/webawesome/dist/components/button/button.js';
	import { m } from '$lib/paraglide/messages.js';
	import { mdiArrowBack } from '../../utils/icon';

	const contactsStore: ContactsStore = getContext('contacts-store');

	const incomingContactRequests = useReactivePromise(
		contactsStore.incomingContactRequests,
	);
	const contacts = useReactivePromise(contactsStore.profilesForAllContacts);

	function rejectContactRequest(contactRequestId: ContactRequestId) {}

	function acceptContactRequest(contactRequestId: ContactRequestId) {}
</script>

<div class="column">
	<div class="top-bar">
		<wa-button class="circle" appearance="plain" href="/">
			<wa-icon src={wrapPathInSvg(mdiArrowBack)}> </wa-icon>
		</wa-button>

		<span class="title">{m.myContacts()}</span>

		<div style="flex: 1"></div>
		<wa-button class="circle" href="/add-contact" appearance="plain">
			<wa-icon src={wrapPathInSvg(mdiAccountPlus)}> </wa-icon>
		</wa-button>
	</div>

	{#await $incomingContactRequests then incomingContactRequests}
		{#if incomingContactRequests.length > 0}
			<wa-card class="center-in-desktop" style="margin: var(--wa-space-m)">
				<div class="column" style="gap: var(--wa-space-m)">
					<span class="title">{m.contactRequests()}</span>

					{#each incomingContactRequests as incomingContactRequest}
						<div
							class="row"
							style="gap: var(--wa-space-s); align-items: center"
						>
							<wa-avatar
								image={incomingContactRequest.profile.avatar}
								initials={incomingContactRequest.profile.name.slice(0, 2)}
							>
							</wa-avatar>
							<span>
								{incomingContactRequest.profile.name}
							</span>
							<div style="flex: 1"></div>

							<wa-button
								variant="danger"
								onclick={async (e: Event) => {
									const button = e.target as WaButton;
									button.loading = true;

									try {
										await rejectContactRequest(
											incomingContactRequest.contactRequestId,
										);
									} catch (e) {}

									button.loading = false;
								}}
								>{m.reject()}
							</wa-button>

							<wa-button
								variant="brand"
								onclick={async (e: Event) => {
									const button = e.target as WaButton;
									button.loading = true;

									try {
										await acceptContactRequest(
											incomingContactRequest.contactRequestId,
										);
									} catch (e) {}

									button.loading = false;
								}}
								>{m.accept()}
							</wa-button>
						</div>
					{/each}
				</div>
			</wa-card>
		{/if}
	{/await}

	{#await $contacts then contacts}
		<wa-card class="center-in-desktop" style="margin: var(--wa-space-m)">
			<div class="column" style="gap: var(--wa-space-m)">
				{#each contacts as [actorId, profile]}
					<wa-button
						appearance="plain"
						class="fill button-with-avatar"
						href={`/direct-messages/${actorId}`}
					>
						<wa-avatar
							slot="start"
							image={profile.avatar}
							initials={profile.name.slice(0, 2)}
						>
						</wa-avatar>

						{profile.name}
					</wa-button>
				{:else}<span>{m.noContactsYet()}</span>
				{/each}
			</div>
		</wa-card>
	{/await}
</div>
