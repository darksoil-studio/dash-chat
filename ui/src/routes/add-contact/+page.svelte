<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/qr-code/qr-code.js';
	import '@awesome.me/webawesome/dist/components/copy-button/copy-button.js';
	import { getContext, onMount } from 'svelte';
	import {
		decodeContactCode,
		encodeContactCode,
		type ContactsStore,
	} from 'dash-chat-stores';
	import { wrapPathInSvg } from '$lib/utils/icon';
	import { mdiQrcode } from '@mdi/js';
	import { m } from '$lib/paraglide/messages.js';

	import { isMobile } from '$lib/utils/environment';
	import { scanQrcode } from '$lib/utils/qrcode';
	import {
		Page,
		Navbar,
		NavbarBackLink,
		Button,
		Link,
		ListInput,
		List,
		Preloader,
	} from 'konsta/svelte';

	const contactsStore: ContactsStore = getContext('contacts-store');

	let code = contactsStore.client.createContactCode().then(encodeContactCode);

	async function receiveCode(code: string) {
		const contactCode = decodeContactCode(code);
		await contactsStore.client.addContact(contactCode);

		// window.history.back();
	}
</script>

<Page>
	<Navbar title={m.addContact()}  titleClass="opacity1" transparent={true}>
		{#snippet left()}
			<NavbarBackLink
				onClick={() => {
					window.history.back();
				}}
			/>
		{/snippet}

		{#snippet right()}
			{#if isMobile}
				<Link
					onClick={async () => {
						try {
							const code = await scanQrcode();
							await receiveCode(code);
						} catch (e) {}
					}}
				>
					<wa-icon src={wrapPathInSvg(mdiQrcode)}> </wa-icon>
				</Link>
			{/if}
		{/snippet}
	</Navbar>

	{#await code}
		<div
			class="column"
			style="height: 100%; align-items: center; justify-content: center"
		>
			<Preloader />
		</div>
	{:then code}
		<div class="column" style="flex:1">
			<div class="column center-in-desktop gap-4 m-6">
				<span>{m.shareThisCode()}</span>

				<wa-qr-code value={code} size="300" style="align-self: center"
				></wa-qr-code>

				<div class="row gap-2" style="align-items: center">
					{code.slice(0, 15)}...
					<wa-copy-button value={code}> </wa-copy-button>
				</div>

				<div class="column gap-1">
					<span>{m.enterYourContactsCode()}</span>

					<List nested>
						<ListInput
							type="text"
							outline
							onInput={(e: Event) => {
								const target = e.target as HTMLInputElement;
								if (target.value) receiveCode(target.value);
							}}
						/>
					</List>
				</div>
			</div>
		</div>
	{/await}
</Page>
