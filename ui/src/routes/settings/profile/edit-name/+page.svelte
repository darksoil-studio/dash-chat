<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/avatar/avatar.js';
	import type { ContactsStore } from 'dash-chat-stores';
	import { getContext } from 'svelte';
	import { goto } from '$app/navigation';
	import { useReactivePromise } from '$lib/stores/use-signal';
	import SelectAvatar from '$lib/components/profiles/SelectAvatar.svelte';
	import { wrapPathInSvg } from '$lib/utils/icon';
	import { mdiClose, mdiContentSave } from '@mdi/js';
	import { editProfile, m } from '$lib/paraglide/messages.js';
	import {
		Button,
		Card,
		Link,
		Navbar,
		NavbarBackLink,
		Page,
		Preloader,
		ListInput,
		List,
		useTheme,
	} from 'konsta/svelte';

	const contactsStore: ContactsStore = getContext('contacts-store');
	let avatar = $state<string | undefined>(undefined);
	let name = $state<string>('');

	const myProfile = useReactivePromise(contactsStore.myProfile);
	myProfile.subscribe(m => {
		m.then(myProfile => {
			if (!name) name = myProfile?.name || '';
			if (!avatar) avatar = myProfile?.avatar;
		});
	});

	async function save() {
		await contactsStore.client.setProfile({
			name: name!,
			avatar,
		});
		goto('/settings/profile');
	}
	const theme = $derived(useTheme());
</script>

<Page>
	{#await $myProfile}
		<div
			class="column"
			style="height: 100%; align-items: center; justify-content: center"
		>
			<Preloader />
		</div>
	{:then myProfile}
		<Navbar
			title={m.editName()}
			titleClass="opacity1"
			transparent={true}
			rightClass={myProfile?.name === name
				? 'pointer-events-none opacity-50'
				: ''}
		>
			{#snippet left()}
				<NavbarBackLink
					onClick={() => goto('/settings/profile')}
				/>
			{/snippet}

			{#snippet right()}
				{#if theme === 'ios'}
					<Link onClick={save}>
						{m.save()}
					</Link>
				{/if}
			{/snippet}
		</Navbar>

		<div class="column">
			<List
				class="center-in-desktop"
				insetIos
				strongIos
				nested={theme === 'material'}
			>
				<ListInput
					type="text"
					outline={theme === 'material'}
					bind:value={name}
					label={theme === 'material' ? m.name() : ''}
					placeholder={theme === 'ios' ? m.name() : ''}
				/>
			</List>
		</div>

		{#if theme === 'material'}
			<Button
				onClick={save}
				class="end-4 bottom-4"
				style="position: fixed; width: auto"
				rounded
				disabled={myProfile?.name === name}
			>
				{m.save()}
			</Button>
		{/if}
	{/await}
</Page>
