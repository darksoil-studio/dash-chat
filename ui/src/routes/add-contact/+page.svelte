<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/button/button.js';
	import '@awesome.me/webawesome/dist/components/qr-code/qr-code.js';
	import '@awesome.me/webawesome/dist/components/copy-button/copy-button.js';
	import { getContext, onMount } from 'svelte';
	import { decodeContactCode, encodeContactCode, type ContactsStore } from 'dash-chat-stores';
	import { useReactivePromise } from '../../stores/use-signal';
	import { wrapPathInSvg } from '@darksoil-studio/holochain-elements';
	import { mdiArrowLeft } from '@mdi/js';

	const contactsStore: ContactsStore = getContext('contacts-store');

	let code = contactsStore.client.createContactCode().then(encodeContactCode);

	async function receiveCode(e: InputEvent) {
		const contactCode =decodeContactCode(e.data!)
		await contactsStore.client.addContact(contactCode);

		// window.history.back();
	}
</script>

<div class="column">
	<div class="top-bar">
		<wa-button
			class="circle"
			appearance="plain"
			onclick={() => {
				window.history.back();
			}}
		>
			<wa-icon src={wrapPathInSvg(mdiArrowLeft)}> </wa-icon>
		</wa-button>
	</div>

	<!-- TODO: add waiting skeleton -->
	{#await code then code}
		<div class="column center-in-desktop" style="gap: var(--wa-space-m); ">
			Share this code:

			<wa-qr-code value={code} size="512" style="align-self: center"
			></wa-qr-code>

			<div class="row" style="gap: var(--wa-space-s); align-items: center">
				{code.slice(0, 10)}...
				<wa-copy-button value={code}> </wa-copy-button>
			</div>

			Enter your contact's code:

			<wa-input oninput={receiveCode}> </wa-input>
		</div>
	{/await}
</div>
