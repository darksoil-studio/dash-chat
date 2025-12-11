<script lang="ts">
	import '@awesome.me/webawesome/dist/components/spinner/spinner.js';
	import type { ContactsStore } from 'dash-chat-stores';
	import { getContext } from 'svelte';
	import { useReactivePromise } from '../../stores/use-signal';
	import { mdiArrowLeft, mdiPencil } from '@mdi/js';
	import { wrapPathInSvg } from '@darksoil-studio/holochain-elements';
	import { m } from '$lib/paraglide/messages.js';

	const contactsStore: ContactsStore = getContext('contacts-store');

	const myProfile = useReactivePromise(contactsStore.myProfile);
</script>

<div class="top-bar">
	<wa-button class="circle" href="/" appearance="plain">
			<wa-icon src={wrapPathInSvg(mdiArrowLeft)}> </wa-icon>
	</wa-button>

	<span class="title">{m.myProfile()}</span>

	<div style="flex: 1"></div>
	<wa-button href="/my-profile/edit" appearance="plain">
		<wa-icon slot="start" src={wrapPathInSvg(mdiPencil)}> </wa-icon>
		Edit
	</wa-button>
</div>

{#await $myProfile}
	<wa-spinner> </wa-spinner>
{:then myProfile}
	<wa-card class="center-in-desktop" style="margin: var(--wa-space-m);">
		<div class="row" style="align-items: center; gap: var(--wa-space-s)">
			<wa-avatar
				image={myProfile?.avatar}
				initials={myProfile?.name.slice(0, 2)}
			>
			</wa-avatar>
			{myProfile?.name}
		</div>
	</wa-card>
{/await}
