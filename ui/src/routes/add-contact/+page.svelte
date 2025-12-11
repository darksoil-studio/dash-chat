<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/button/button.js';
	import '@awesome.me/webawesome/dist/components/qr-code/qr-code.js';
	import '@awesome.me/webawesome/dist/components/copy-button/copy-button.js';
	import { getContext, onMount } from 'svelte';
	import {
		decodeContactCode,
		encodeContactCode,
		type ContactsStore,
	} from 'dash-chat-stores';
	import { wrapPathInSvg } from '@darksoil-studio/holochain-elements';
	import { mdiArrowLeft, mdiQrcode } from '@mdi/js';
	import { m } from '$lib/paraglide/messages.js';

	import { isMobile } from '../../utils/environment';
	import { scanQrcode } from '../../utils/qrcode';

	const contactsStore: ContactsStore = getContext('contacts-store');

	let code = contactsStore.client.createContactCode().then(encodeContactCode);

	async function receiveCode(code: string) {
		const contactCode = decodeContactCode(code);
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
		<span class="title">{m.addContact()}</span>

		<div style="flex:1"></div>

		{#if isMobile}
			<wa-button
				appearance="plain"
				onclick={async () => {
					try {
						const code = await scanQrcode();
						await receiveCode(code);
					} catch (e) {}
				}}
			>
				<wa-icon slot="start" src={wrapPathInSvg(mdiQrcode)}> </wa-icon>

				Scan
			</wa-button>
		{/if}
	</div>

	{#await code then code}
		<div class="column center-in-desktop" style="gap: var(--wa-space-m); margin: var(--wa-space-m); ">
			{m.shareThisCode()}

			<wa-qr-code value={code} size="300" style="align-self: center"
			></wa-qr-code>

			<div class="row" style="gap: var(--wa-space-s); align-items: center">
				{code.slice(0, 15)}...
				<wa-copy-button value={code}> </wa-copy-button>
			</div>

			{m.enterYourContactsCode()}

			<wa-input oninput={(e: InputEvent) => receiveCode(e.data!)}> </wa-input>
		</div>
	{/await}
</div>
