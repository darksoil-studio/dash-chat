<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/avatar/avatar.js';
	import type { ContactsStore } from 'dash-chat-stores';
	import { getContext } from 'svelte';
	import { useReactivePromise } from '../../stores/use-signal';
	import { mdiPencil } from '@mdi/js';
	import { wrapPathInSvg } from '@darksoil-studio/holochain-elements';
	import { m } from '$lib/paraglide/messages.js';
	import {
		Card,
		Link,
		Navbar,
		NavbarBackLink,
		Page,
		Preloader,
	} from 'konsta/svelte';

	const contactsStore: ContactsStore = getContext('contacts-store');

	const myProfile = useReactivePromise(contactsStore.myProfile);
</script>

<Page>
	<Navbar title={m.myProfile()}>
		{#snippet left()}
			<NavbarBackLink onClick={() => (window.location.href = '/')} />
		{/snippet}

		{#snippet right()}
			<Link href="/my-profile/edit">
				<wa-icon src={wrapPathInSvg(mdiPencil)}> </wa-icon>
				{m.edit()}
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
			<Card class="center-in-desktop" raised>
				<div class="row gap-4" style="align-items: center;">
					<wa-avatar
						image={myProfile?.avatar}
						initials={myProfile?.name.slice(0, 2)}
					>
					</wa-avatar>
					{myProfile?.name}
				</div>
			</Card>
		</div>
	{/await}
</Page>
