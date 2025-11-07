<script lang="ts">
	import '@awesome.me/webawesome/dist/components/spinner/spinner.js';
	import type { UsersStore } from 'dash-chat-stores';
	import { getContext } from 'svelte';
	import { useReactivePromise } from '../../../stores/use-signal';
	import WaInput from '@awesome.me/webawesome/dist/components/input/input.js';
	import SelectAvatar from '../../../components/SelectAvatar.svelte';

	const usersStore: UsersStore = getContext('users-store');
	let avatar: string | undefined;
	let name: string | undefined;

	const me = useReactivePromise(usersStore.me);
	me.subscribe(m => {
		m.then(me => {
			if (!name) {
				name = me?.profile?.name;
			}
			if (!avatar) {
				avatar = me?.profile?.avatar;
			}
		});
	});

	async function setProfile() {
		await usersStore.client.setProfile({
			name: name!,
			avatar,
		});
		window.location.href = '/my-profile';
	}
</script>

<div class="top-bar">
	<wa-button class="circle" href="/my-profile" appearance="plain">
		<wa-icon name="close"> </wa-icon>
	</wa-button>

	<span class="title">Edit profile</span>

	<div style="flex: 1"></div>
	<wa-button appearance="plain" onclick={setProfile}>
		<wa-icon slot="start" name="floppy-disk"> </wa-icon>
		Save
	</wa-button>
</div>

{#await $me}
	<wa-spinner> </wa-spinner>
{:then me}
	<wa-card style="margin: var(--wa-space-m)">
		<div class="row" style="align-items: center; gap: var(--wa-space-s)">
			<SelectAvatar bind:avatar></SelectAvatar>

			<wa-input
				style="flex: 1"
				defaultValue={me?.profile?.name}
				oninput={(e: CustomEvent) => {
					name = (e.target as WaInput).value!;
				}}
			>
			</wa-input>
		</div>
	</wa-card>
{/await}
