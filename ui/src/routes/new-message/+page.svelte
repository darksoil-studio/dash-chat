<script lang="ts">
	import '@awesome.me/webawesome/dist/components/button/button.js';
	import { m } from '$lib/paraglide/messages.js';
	import {
		mdiAccountMultiplePlus,
		mdiAccountPlus,
	} from '@mdi/js';
	import type { ContactsStore } from 'dash-chat-stores';
	import { getContext } from 'svelte';
	import { useReactivePromise } from '../../stores/use-signal';
	import { mdiArrowBack, wrapPathInSvg } from '../../utils/icon';

	const contactsStore: ContactsStore = getContext('contacts-store');

	const contacts = useReactivePromise(contactsStore.profilesForAllContacts);
</script>

<div class="top-bar">
	<wa-button class="circle" href="/" appearance="plain">
		<wa-icon src={wrapPathInSvg(mdiArrowBack)}> </wa-icon>
	</wa-button>
	<span class="title">{m.newMessage()}</span>
</div>

<div
	class="column center-in-desktop"
	style="gap: var(--wa-space-l); margin: var(--wa-space-m)"
>
	<div class="column" style="gap: var(--wa-space-m)">
		<wa-button appearance="outlined" href="/add-contact">
			<wa-icon slot="start" src={wrapPathInSvg(mdiAccountPlus)}> </wa-icon>
			{m.addContact()}
		</wa-button>

		<wa-button appearance="outlined" href="/new-group">
			<wa-icon slot="start" src={wrapPathInSvg(mdiAccountMultiplePlus)}>
			</wa-icon>
			{m.newGroup()}
		</wa-button>
	</div>

	<div class="column" style="gap: var(--wa-space-m)">
		<span class="title">{m.contacts()}</span>

		{#await $contacts then contacts}
			<div class="column" style="gap: var(--wa-space-m)">
				{#each contacts as [actorId, profile]}
					<wa-button
						appearance="plain"
						class="button-with-avatar fill"
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
				{:else}
					<span>{m.noContactsYet()}</span>
				{/each}
			</div>
		{/await}
	</div>
</div>
