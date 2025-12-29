<script lang="ts">
	import { getContext } from 'svelte';
	import type { ContactsStore } from 'dash-chat-stores';
	import SelectAvatar from '../components/SelectAvatar.svelte';
	import { m } from '$lib/paraglide/messages.js';
	import {
		Page,
		Button,
		ListInput,
		List,
		BlockTitle,
		useTheme,
		Link,
		Navbar,
	} from 'konsta/svelte';

	const contactsStore: ContactsStore = getContext('contacts-store');
	let nickname = $state<string | undefined>(undefined);
	let avatar = $state<string | undefined>(undefined);

	async function setProfile() {
		await contactsStore.client.setProfile({
			name: nickname!,
			avatar,
		});
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
</Page>
