<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/button/button.js';
	import type { ContactsStore } from 'dash-chat-stores';
	import { getContext } from 'svelte';
	import { useReactivePromise } from '../stores/use-signal';
	import Avatar from '../components/Avatar.svelte';
	import { wrapPathInSvg } from '@darksoil-studio/holochain-elements';
	import { mdiAccountGroup, mdiSquareEditOutline } from '@mdi/js';

	const contactsStore: ContactsStore = getContext('contacts-store');

	const myPubKey = useReactivePromise(contactsStore.myPubKey);
</script>

{#await $myPubKey then myPubKey}
	<div class="column" style="flex: 1">
		<div class="top-bar">
			<a href="/my-profile">
				<Avatar publicKey={myPubKey}></Avatar>
			</a>
			<div style="flex: 1"></div>

			<wa-button class="circle" href="/contacts" appearance="plain">
				<wa-icon src={wrapPathInSvg(mdiAccountGroup)}> </wa-icon>
			</wa-button>

			<wa-button class="circle" href="/new-message" appearance="plain">
				<wa-icon src={wrapPathInSvg(mdiSquareEditOutline)}> </wa-icon>
			</wa-button>
		</div>
	</div>
{/await}

<style>
</style>
