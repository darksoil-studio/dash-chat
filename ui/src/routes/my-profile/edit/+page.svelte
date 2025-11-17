<script lang="ts">
	import '@awesome.me/webawesome/dist/components/spinner/spinner.js';
	import type { ContactsStore } from 'dash-chat-stores';
	import { getContext } from 'svelte';
	import { useReactivePromise } from '../../../stores/use-signal';
	import WaInput from '@awesome.me/webawesome/dist/components/input/input.js';
	import SelectAvatar from '../../../components/SelectAvatar.svelte';
	import { wrapPathInSvg } from '@darksoil-studio/holochain-elements';
	import { mdiClose, mdiContentSave } from '@mdi/js';

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

<div class="top-bar">
	<wa-button class="circle" href="/my-profile" appearance="plain">
		<wa-icon src={wrapPathInSvg(mdiClose)}> </wa-icon>
	</wa-button>

	<span class="title">Edit profile</span>

	<div style="flex: 1"></div>
	<wa-button appearance="plain" onclick={setProfile}>
		<wa-icon slot="start" src={wrapPathInSvg(mdiContentSave)}> </wa-icon>
		Save
	</wa-button>
</div>

{#await $myProfile}
	<wa-spinner> </wa-spinner>
{:then myProfile}
	<wa-card style="margin: var(--wa-space-m)">
		<div class="row" style="align-items: center; gap: var(--wa-space-s)">
			<SelectAvatar bind:value={avatar} defaultValue={myProfile?.avatar}></SelectAvatar>

			<wa-input
				style="flex: 1"
				defaultValue={myProfile?.name}
				oninput={(e: InputEvent) => {
					name = (e.target as WaInput).value!;
				}}
			>
			</wa-input>
		</div>
	</wa-card>
{/await}
