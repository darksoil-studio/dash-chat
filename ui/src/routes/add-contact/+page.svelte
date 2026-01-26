<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/qr-code/qr-code.js';
	import '@awesome.me/webawesome/dist/components/copy-button/copy-button.js';
	import { getContext } from 'svelte';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import {
		decodeContactCode,
		encodeContactCode,
		type ContactsStore,
	} from 'dash-chat-stores';
	import type { AddContactError } from 'dash-chat-stores';
	import { wrapPathInSvg } from '$lib/utils/icon';
	import { mdiContentCopy, mdiQrcode } from '@mdi/js';
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
		Card,
		Preloader,
		Toast,
		Button,
	} from 'konsta/svelte';
	import { goto } from '$app/navigation';
	import { TOAST_TTL_MS } from '$lib/utils/toasts';

	const contactsStore: ContactsStore = getContext('contacts-store');

	let myCode = contactsStore.client.createContactCode().then(encodeContactCode);

	let contactAlreadyExistsToastOpen = $state(false);
	let copiedToastOpen = $state(false);
	let errorMessage = $state<string | undefined>(undefined);
	let t: NodeJS.Timeout | undefined;

	async function receiveCode(code: string) {
		try {
			const contactCode = decodeContactCode(code);

			const myCodeString = await myCode;

			if (code === myCodeString) {
				errorMessage = m.cantAddYourselfAsContact();
				clearTimeout(t);
				t = setTimeout(() => {
					errorMessage = undefined;
				}, TOAST_TTL_MS);
				return;
			}

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
			console.error(e);
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
			clearTimeout(t);
			t = setTimeout(() => {
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
						} catch (e) {
							console.error(e);
							errorMessage = m.errorScanningQrCode();
							clearTimeout(t);
							t = setTimeout(() => {
								errorMessage = undefined;
							}, TOAST_TTL_MS);
						}
					}}
				>
					<wa-icon src={wrapPathInSvg(mdiQrcode)}> </wa-icon>
				</Link>
			{/if}
		{/snippet}
	</Navbar>

	{#await myCode}
		<div
			class="column"
			style="height: 100%; align-items: center; justify-content: center"
		>
			<Preloader />
		</div>
	{:then code}
		<div class="column" style="flex:1">
			<div class="column center-in-desktop gap-4 m-6">
				<Card class="qr-card p-2 pb-0">
					<div class="column gap-2" style="align-items: center">
						<div
							class="column p-2"
							style="align-items: center; justify-content: center; background-color: white; border-radius: 8px;"
						>
							<wa-qr-code value={code} size="250" fill="#007aff"></wa-qr-code>
						</div>

						<div>
							<Button
								colors={{
									touchRipple: 'white',
									textIos: 'text-white',
									textMaterial: 'text-white',
								}}
								clearMaterial
								onClick={async () => {
									await writeText(code);
									copiedToastOpen = true;
									clearTimeout(t);
									t = setTimeout(() => {
										copiedToastOpen = false;
									}, TOAST_TTL_MS);
								}}
							>
								<wa-icon src={wrapPathInSvg(mdiContentCopy)}> </wa-icon>

								{code.slice(0, 15)}...
							</Button>
						</div>
					</div>
				</Card>
				<span class="mx-6 mb-2">{m.shareCodeWarning()}</span>

				<div class="column gap-1">
					<List nested strongIos insetIos>
						<ListInput
							floatingLabel
							label={m.enterYourContactsCode()}
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
	<Toast position="center" opened={copiedToastOpen}
		>{m.copiedCodeToClipboard()}</Toast
	>
	<Toast
		position="center"
		class="k-color-brand-red"
		opened={errorMessage !== undefined}>{errorMessage}</Toast
	>
</Page>

<style>
	:global(.qr-card) {
		background-color: var(--color-brand-primary);
		align-self: center;
	}
</style>
