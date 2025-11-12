<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/button/button.js';
	import '@awesome.me/webawesome/dist/components/copy-button/copy-button.js';
	import { getContext } from 'svelte';
	import type { ContactsStore } from 'dash-chat-stores';
	import { useReactivePromise } from '../../stores/use-signal';
	import { wrapPathInSvg } from '@darksoil-studio/holochain-elements';
	import { mdiArrowLeft } from '@mdi/js';

	const contactsStore: ContactsStore = getContext('contacts-store');

	const myMemberCode = useReactivePromise(contactsStore.myMemberCode);

	async function receiveCode(e: InputEvent) {
		await contactsStore.client.addContact(e.data!);

		window.history.back();
	}
</script>

<div class="column">
	<div class="top-bar">
		<wa-button
			class="circle"
			appearance="plain"
			onclick={() => {
				window.history.back();
			}}
		>
			<wa-icon src={wrapPathInSvg(mdiArrowLeft)}> </wa-icon>
		</wa-button>
	</div>

	<!-- TODO: add waiting skeleton -->
	{#await $myMemberCode then memberCode}
		<div class="column center-in-desktop" style="gap: var(--wa-space-m)">
			Share this code:
			<div class="row" style="gap: var(--wa-space-s); align-items: center">
				{memberCode.slice(0, 10)}...
				<wa-copy-button value={memberCode}> </wa-copy-button>
			</div>

			Enter your contact's code:

			<wa-input oninput={receiveCode}> </wa-input>
		</div>
	{/await}
</div>
