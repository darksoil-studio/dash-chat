<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/button/button.js';
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
		<div style="flex: 1"></div>
		<wa-button class="circle" href="/add-contact" appearance="plain">
			<wa-icon src={wrapPathInSvg(mdiAccountPlus)}> </wa-icon>
		</wa-button>
	</div>

	{#await $contacts then contacts}
		<wa-card class="center-in-desktop">
			<div class="column" style="gap: var(--wa-space-m)">
				{#each contacts as [publicKey, profile]}
					<div class="row" style="gap: var(--wa-space-s); align-items: center">
						<Avatar {publicKey}></Avatar>

						{profile.name}
					</div>{:else}<span>You don't have any contacts yet.</span>
				{/each}
			</div>
		</wa-card>
	{/await}
</div>
