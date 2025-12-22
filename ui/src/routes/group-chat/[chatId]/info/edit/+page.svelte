<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/avatar/avatar.js';
	import { m } from '$lib/paraglide/messages.js';

	import { useReactivePromise } from '../../../../../stores/use-signal';
	import { getContext } from 'svelte';
	import type { ContactsStore, ChatsStore, PublicKey } from 'dash-chat-stores';
	import { wrapPathInSvg } from '@darksoil-studio/holochain-elements';
	import {
		mdiAccountGroup,
		mdiClose,
		mdiContentSave,
		mdiPencil,
		mdiSend,
	} from '@mdi/js';
	import SelectAvatar from '../../../../../components/SelectAvatar.svelte';
	import {
		Page,
		Navbar,
		NavbarBackLink,
		Link,
		Card,
		ListInput,
		List,
	} from 'konsta/svelte';

	const chatId = window.location.href.split('/').reverse()[2];

	const chatsStore: ChatsStore = getContext('chats-store');
	const store = chatsStore.groupChats(chatId);
	const info = useReactivePromise(store.info);

	let avatar: string | undefined;
	let name = $state<string>('');
	let description = $state<string>('');

	info.subscribe(i => {
		i.then(info => {
			if (!avatar) avatar = info?.avatar;
			if (!name) name = info?.name || '';
			if (!description) description = info?.description || '';
		});
	});

	async function save() {
		window.location.href = `/group-chat/${chatId}/info`;
	}
</script>

<Page>
	<Navbar title={m.editGroup()}  titleClass="opacity1" transparent={true}>
		{#snippet left()}
			<NavbarBackLink
				onClick={() => (window.location.href = `/group-chat/${chatId}/info`)}
			/>
		{/snippet}

		{#snippet right()}
			<Link onClick={save}>
				<wa-icon src={wrapPathInSvg(mdiContentSave)}> </wa-icon>
				{m.save()}
			</Link>
		{/snippet}
	</Navbar>

	{#await $info then info}
		<div class="column m-2">
			<div class="column center-in-desktop">
				<SelectAvatar defaultValue={info.avatar} bind:value={avatar}
				></SelectAvatar>

				<List nested>
					<ListInput type="text" outline bind:value={name} label={m.name()} />

					<ListInput
						type="textarea"
						outline
						inputStyle={{"min-height": "2em"}}
						bind:value={description}
						label={m.description()}
					/>
				</List>
			</div>
		</div>
	{/await}
</Page>
