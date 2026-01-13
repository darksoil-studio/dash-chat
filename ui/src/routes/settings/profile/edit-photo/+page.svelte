<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/avatar/avatar.js';
	import type { ContactsStore } from 'dash-chat-stores';
	import { getContext } from 'svelte';
	import { useReactivePromise } from '$lib/stores/use-signal';
	import SelectAvatar from '$lib/components/profiles/SelectAvatar.svelte';
	import { wrapPathInSvg } from '$lib/utils/icon';
	import { mdiClose, mdiContentSave, mdiImage } from '@mdi/js';
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
	import { resizeAndExport } from '$lib/utils/image';

	const contactsStore: ContactsStore = getContext('contacts-store');
	let avatar = $state<string | undefined>(undefined);
	let name = $state<string>('');

	const myProfile = useReactivePromise(contactsStore.myProfile);
	myProfile.subscribe(m => {
		m.then(myProfile => {
			if (!name) name = myProfile?.name || '';
			if (!avatar) avatar = myProfile?.avatar;
			console.log('aa', myProfile)
		});
	});

	async function save() {
		await contactsStore.client.setProfile({
			name: name!,
			avatar,
		});
		window.location.href = '/settings/profile';
	}
	const theme = $derived(useTheme());

	
	let avatarFilePicker: HTMLInputElement;
	function onAvatarUploaded() {
		if (avatarFilePicker.files && avatarFilePicker.files[0]) {
			const reader = new FileReader();
			reader.onload = e => {
				const img = new Image();
				img.crossOrigin = 'anonymous';
				img.onload = () => {
					avatar = resizeAndExport(img);
					avatarFilePicker.value = '';
				};
				img.src = e.target?.result as string;
			};
			reader.readAsDataURL(avatarFilePicker.files[0]);
		}
	}
</script>

<input
	type="file"
	bind:this={avatarFilePicker}
	style="display: none"
	onchange={onAvatarUploaded}
/>

<Page>

	{#await $myProfile}
		<div
			class="column"
			style="height: 100%; align-items: center; justify-content: center"
		>
			<Preloader />
		</div>
	{:then myProfile}
	<Navbar titleClass="opacity1" transparent={true}
		rightClass={myProfile?.avatar === avatar
			? 'pointer-events-none opacity-50'
			: ''}
	>
		{#snippet left()}
			<NavbarBackLink
				onClick={() => (window.location.href = '/settings/profile')}
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
		<div class="column" style="flex: 1">
			<div class="center-in-desktop gap-4"  style="align-items: center;">
				<div class="column m-10 gap-1" style="align-items: center;">
					<wa-avatar style="--size: 100px" image={avatar}>
					</wa-avatar>
				</div>

				<div class="row gap-2" style="flex: 1; justify-content: center;">
					<div class="column"  style="align-items: center;">
						<Button rounded clear onClick={()=>avatarFilePicker.click()}>
							<wa-icon src={wrapPathInSvg(mdiImage)}> </wa-icon>
						</Button>
						<span>{m.photo()} </span>
					</div>
				</div>
			</div>
		</div>

		{#if theme === 'material'}
			<Button
				onClick={save}
				class="end-4 bottom-4"
				style="position: fixed; width: auto"
				rounded
				disabled={myProfile?.avatar === avatar}
			>
				{m.save()}
			</Button>
		{/if}
	{/await}
</Page>
