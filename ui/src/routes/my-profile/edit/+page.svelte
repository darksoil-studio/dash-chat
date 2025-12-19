<script lang="ts">
	import '@awesome.me/webawesome/dist/components/spinner/spinner.js';
	import type { ContactsStore } from 'dash-chat-stores';
	import { getContext } from 'svelte';
	import { useReactivePromise } from '../../../stores/use-signal';
	import WaInput from '@awesome.me/webawesome/dist/components/input/input.js';
	import SelectAvatar from '../../../components/SelectAvatar.svelte';
	import { wrapPathInSvg } from '@darksoil-studio/holochain-elements';
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
	} from 'konsta/svelte';

	const contactsStore: ContactsStore = getContext('contacts-store');
	let avatar: string | undefined;
	let name: string | undefined;

	const myProfile = useReactivePromise(contactsStore.myProfile);
	myProfile.subscribe(m => {
		m.then(myProfile => {
			if (!name) {
				name = myProfile?.name;
			}
			if (!avatar) {
				avatar = myProfile?.avatar;
			}
		});
	});

	async function setProfile() {
		await contactsStore.client.setProfile({
			name: name!,
			avatar,
		});
		window.location.href = '/my-profile';
	}
</script>

<Page>
	<Navbar title={m.editProfile()}>
		{#snippet left()}
			<NavbarBackLink onClick={() => (window.location.href = '/my-profile')} />
		{/snippet}

		{#snippet right()}
			<Link onClick={setProfile}>
				<wa-icon src={wrapPathInSvg(mdiContentSave)}> </wa-icon>
				{m.save()}
			</Link>
		{/snippet}
	</Navbar>

	{#await $myProfile}
		<div
			class="column"
			style="height: 100%; align-items: center; justify-content: center"
		>
			<Preloader />
		</div>
	{:then myProfile}
		<div class="column" style="flex: 1">
			<Card class="center-in-desktop" style="margin: var(--wa-space-m);">
				<div class="row" style="align-items: center; gap: var(--wa-space-s)">
					<SelectAvatar bind:value={avatar} defaultValue={myProfile?.avatar}
					></SelectAvatar>

					<wa-input
						style="flex: 1"
						defaultValue={myProfile?.name}
						oninput={(e: InputEvent) => {
							name = (e.target as WaInput).value!;
						}}
					>
					</wa-input>
				</div>
			</Card>
		</div>
	{/await}
</Page>
