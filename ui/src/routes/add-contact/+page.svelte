<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/qr-code/qr-code.js';
	import '@awesome.me/webawesome/dist/components/copy-button/copy-button.js';
	import { getContext } from 'svelte';
	import {
		toPromise,
		decodeContactCode,
		encodeContactCode,
		type ContactsStore,
	} from 'dash-chat-stores';
	import type { AddContactError } from 'dash-chat-stores';
	import { wrapPathInSvg } from '$lib/utils/icon';
	import { mdiQrcode } from '@mdi/js';
	import { m } from '$lib/paraglide/messages.js';

	import { isMobile } from '$lib/utils/environment';
	import { scanQrcode } from '$lib/utils/qrcode';
	import {
		Page,
		Navbar,
		NavbarBackLink,
		Link,
		ListInput,
		List,
		Preloader,
		Toast,
	} from 'konsta/svelte';
	import { goto } from '$app/navigation';
	import { TOAST_TTL_MS } from '$lib/utils/toasts';

	const contactsStore: ContactsStore = getContext('contacts-store');

	let code = contactsStore.client.createContactCode().then(encodeContactCode);

	let contactAlreadyExistsToastOpen = $state(false);
	let errorMessage = $state<string | undefined>(undefined);
	let t: NodeJS.Timeout | undefined;

	async function receiveCode(code: string) {
		try {
			const contactCode = decodeContactCode(code);

			// Don't send a contact request if they're already in your contacts
			// 
			// Uncommenting this would mean that if the contact rejected your contact request
			// there is no way to resend the contact request
			//
			// const contacts = await toPromise(contactsStore.contactsAgentIds);
			// 
			// if (contacts.includes(contactCode.agent_id)) {
			// 	contactAlreadyExistsToastOpen = true;
			// 	t = setTimeout(() => {
			// 		clearTimeout(t);
			// 		contactAlreadyExistsToastOpen = false;
			// 	}, TOAST_TTL_MS);
			// 	return;
			// }

			await contactsStore.client.addContact(contactCode);

			goto(`/direct-messages/${contactCode.agent_id}`);
		} catch (e) {
			const error = e as AddContactError;
			switch (error.kind) {
				case 'ProfileNotCreated':
					errorMessage = m.errorAddContactProfileRequired();
					break;
				case 'InitializeTopic':
				case 'AuthorOperation':
				case 'CreateQrCode':
				case 'CreateDirectChat':
					errorMessage = m.errorAddContact();
					break;
				default:
					errorMessage = m.errorUnexpected();
			}
			t = setTimeout(() => {
				clearTimeout(t);
				errorMessage = undefined;
			}, TOAST_TTL_MS);
		}
	}
</script>

<Page>
	<Navbar title={m.addContact()} titleClass="opacity1" transparent={true}>
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
							onInput={async (e: Event) => {
								const target = e.target as HTMLInputElement;
								if (target.value) {
									await receiveCode(target.value);
									target.value = '';
								}
							}}
						/>
					</List>
				</div>
			</div>
		</div>
	{/await}
	<Toast position="center" opened={contactAlreadyExistsToastOpen}
		>{m.contactAlreadyExists()}</Toast
	>
	<Toast
		position="center"
		class="k-color-brand-red"
		opened={errorMessage !== undefined}>{errorMessage}</Toast
	>
</Page>
