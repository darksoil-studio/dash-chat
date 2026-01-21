<script lang="ts">
	import { getContext } from 'svelte';
	import type { ContactsStore, Error } from 'dash-chat-stores';
	import SelectAvatar from './SelectAvatar.svelte';
	import { m } from '$lib/paraglide/messages.js';
	import { TOAST_TTL_MS } from '$lib/utils/toasts';
	import {
		Page,
		Button,
		ListInput,
		List,
		useTheme,
		Link,
		Navbar,
		Toast,
	} from 'konsta/svelte';

	const contactsStore: ContactsStore = getContext('contacts-store');
	let nickname = $state<string | undefined>(undefined);
	let avatar = $state<string | undefined>(undefined);
	let errorMessage = $state<string | undefined>(undefined);

	async function setProfile() {
		try {
			await contactsStore.client.setProfile({
				name: nickname!,
				avatar,
			});
		} catch (e) {
			const error = e as Error;
			switch (error.kind) {
				case 'AuthorOperation':
					errorMessage = m.errorSetProfile();
					break;
				default:
					errorMessage = m.errorUnexpected();
			}
			setTimeout(() => {
				errorMessage = undefined;
			}, TOAST_TTL_MS);
		}
	}
	const theme = $derived(useTheme());
</script>

<Page>
	<Navbar
		title={m.createProfile()}
		titleClass="opacity1"
		transparent={true}
		rightClass={nickname === undefined || nickname === ''
			? 'pointer-events-none opacity-50'
			: ''}
	>
		{#snippet right()}
			{#if theme === 'ios'}
				<Link onClick={setProfile}>
					{m.create()}
				</Link>
			{/if}
		{/snippet}
	</Navbar>

	<div class="column" style="flex: 1">
		<div class="center-in-desktop">
			<List nested={theme === 'material'} insetIos strongIos>
				<ListInput
					outline
					class="plain"
					type="text"
					onInput={e => (nickname = e.target.value)}
					label={m.name()}
				>
					{#snippet media()}
						<SelectAvatar bind:value={avatar}></SelectAvatar>
					{/snippet}
				</ListInput>
			</List>
		</div>
	</div>

	{#if theme === 'material'}
		<Button
			onClick={setProfile}
			class="end-4 bottom-4"
			style="position: fixed; width: auto"
			rounded
			disabled={nickname === undefined || nickname === ''}
		>
			{m.create()}
		</Button>
	{/if}
	<Toast position="center" opened={errorMessage !== undefined}
		>{errorMessage}</Toast
	>
</Page>
