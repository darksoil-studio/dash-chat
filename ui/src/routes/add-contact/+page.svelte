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
		Button,
	} from 'konsta/svelte';
	import { goto } from '$app/navigation';
	import { showToast } from '$lib/utils/toasts';
	import { cancel } from '@tauri-apps/plugin-barcode-scanner';

	const contactsStore: ContactsStore = getContext('contacts-store');

	let myCode = contactsStore.client.createContactCode().then(encodeContactCode);

	let tab = $state<'code' | 'scan'>('code');

	async function receiveCode(code: string) {
		try {
			const contactCode = decodeContactCode(code);

			const myCodeString = await myCode;

			if (code === myCodeString) {
				showToast(m.cantAddYourselfAsContact(), 'error');
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
			// 	showToast(m.contactAlreadyExists());
			// 	return;
			// }

			await contactsStore.client.addContact(contactCode);
			showToast(m.contactAccepted())

			goto(`/direct-messages/${contactCode.agent_id}`);

		} catch (e) {
			console.error(e);
			const error = e as AddContactError;
			switch (error.kind) {
				case 'ProfileNotCreated':
					showToast(m.errorAddContactProfileRequired(), 'error');
					break;
				case 'InitializeTopic':
				case 'AuthorOperation':
				case 'CreateQrCode':
				case 'CreateDirectChat':
					showToast(m.errorAddContact(), 'error');
					break;
				default:
					showToast(m.errorUnexpected(), 'error');
			}
		}
	}
</script>

<Page
	class={tab === 'scan' ? 'transparent' : ''}
	style="display: flex; flex-direction: column;"
>
	<Navbar
		centerTitle={isMobile}
		titleClass="opacity1"
		transparent={tab !== 'scan'}
	>
		{#snippet left()}
			<NavbarBackLink
				onClick={() => {
					window.history.back();
				}}
			/>
		{/snippet}

		{#snippet title()}
			{#if isMobile}
				<div
					class="row gap-2"
					style="align-items: center; justify-content: center"
				>
					<Button
						small
						rounded
						tonal={tab !== 'code'}
						onClick={async () => {
							if (tab === 'code') return;
							tab = 'code';
							await cancel();
						}}
						>{m.code()}
					</Button>

					<Button
						small
						rounded
						tonal={tab !== 'scan'}
						onClick={async () => {
							if (tab === 'scan') return;
							tab = 'scan';
							try {
								const code = await scanQrcode();
								await receiveCode(code);
							} catch (e) {
								console.error(e);
								showToast(m.errorScanningQrCode(), 'error');
							}
						}}
						>{m.scan()}
					</Button>
				</div>
			{:else}
				{m.addContact()}
			{/if}
		{/snippet}
	</Navbar>

	{#if tab === 'code'}
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
										showToast(m.copiedCodeToClipboard());
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
	{:else}
		<div class="column" style="flex: 1">
			<div
				class="column"
				style="flex: 1; align-items: center; justify-content: center"
			>
				<div class="barcode-scanner--area--container">
					<div class="square surround-cover">
						<div class="barcode-scanner--area--outer surround-cover"></div>
					</div>
				</div>
			</div>
			<div
				class="row p-2"
				style="background-color: var(--background-color); align-items: center; justify-content: center; z-index: 1"
			>
				<span style="margin-bottom: env(safe-area-inset-bottom)"
					>{m.scanQrCodeOfYourContact()}</span
				>
			</div>
		</div>
	{/if}
</Page>

<style>
	:global(.qr-card) {
		background-color: var(--color-brand-primary);
		align-self: center;
	}

	.square {
		width: 100%;
		position: relative;
		overflow: hidden;
		transition: 0.3s;
	}
	.square:after {
		content: '';
		top: 0;
		display: block;
		padding-bottom: 100%;
	}
	.square > div {
		position: absolute;
		top: 0;
		left: 0;
		bottom: 0;
		right: 0;
	}

	.surround-cover {
		box-shadow: 0 0 0 99999px rgba(0, 0, 0, 0.5);
	}

	.barcode-scanner--area--container {
		width: 80%;
		max-width: min(500px, 80vh);
		margin: auto;
	}
	.barcode-scanner--area--outer {
		display: flex;
		border-radius: 1em;
	}
</style>
