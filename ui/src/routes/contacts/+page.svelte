<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/button/button.js';
	import '@awesome.me/webawesome/dist/components/avatar/avatar.js';
	import { useReactivePromise } from '../../stores/use-signal';
	import { getContext } from 'svelte';
	import type { ContactsStore } from 'dash-chat-stores';
	import Avatar from '../../components/Avatar.svelte';
	import { wrapPathInSvg } from '@darksoil-studio/holochain-elements';
	import { mdiAccountPlus, mdiArrowLeft } from '@mdi/js';

	const contactsStore: ContactsStore = getContext('contacts-store');

	const contacts = useReactivePromise(contactsStore.profilesForAllContacts);
</script>

<div class="column">
	<div class="top-bar">
		<wa-button class="circle" appearance="plain" href="/">
			<wa-icon src={wrapPathInSvg(mdiArrowLeft)}> </wa-icon>
		</wa-button>

		<span class="title">My contacts</span>

		<div style="flex: 1"></div>
		<wa-button class="circle" href="/add-contact" appearance="plain">
			<wa-icon src={wrapPathInSvg(mdiAccountPlus)}> </wa-icon>
		</wa-button>
	</div>

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
				{:else}<span>You don't have any contacts yet.</span>
				{/each}
			</div>
		</wa-card>
	{/await}
</div>
