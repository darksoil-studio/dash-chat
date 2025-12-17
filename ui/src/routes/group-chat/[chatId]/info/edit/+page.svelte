<script lang="ts">
	import '@awesome.me/webawesome/dist/components/button/button.js';
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/avatar/avatar.js';

	import { useReactivePromise } from '../../../../../stores/use-signal';
	import { getContext } from 'svelte';
	import type { ContactsStore, ChatsStore, PublicKey } from 'dash-chat-stores';
	import { wrapPathInSvg } from '@darksoil-studio/holochain-elements';
	import {
		mdiAccountGroup,
		mdiArrowLeft,
		mdiClose,
		mdiContentSave,
		mdiPencil,
		mdiSend,
	} from '@mdi/js';
	import WaInput from '@awesome.me/webawesome/dist/components/input/input.js';
	import WaTextarea from '@awesome.me/webawesome/dist/components/textarea/textarea.js';
	import SelectAvatar from '../../../../../components/SelectAvatar.svelte';

	const chatId = window.location.href.split('/').reverse()[2];

	const chatsStore: ChatsStore = getContext('chats-store');
	const store = chatsStore.groupChats(chatId);
	const info = useReactivePromise(store.info);

	let avatar: string | undefined;
	let name: string | undefined;
	let description: string | undefined;

	info.subscribe(i => {
		i.then(info => {
			if (!avatar) {
				avatar = info?.avatar;
			}
		});
	});
	async function save() {
		window.location.href = `/group-chat/${chatId}/info`;
	}
</script>

<div class="top-bar">
	<wa-button
		class="circle"
		appearance="plain"
		href={`/group-info/${chatId}/info`}
	>
		<wa-icon src={wrapPathInSvg(mdiClose)}> </wa-icon>
	</wa-button>
	<span class="title">Edit group</span>

	<div style="flex: 1"></div>

	<wa-button appearance="plain" onclick={save}>
		<wa-icon slot="start" src={wrapPathInSvg(mdiContentSave)}> </wa-icon>
		Save
	</wa-button>
</div>

{#await $info then info}
	<wa-card class="center-in-desktop" style="margin: var(--wa-space-m)">
		<div class="column" style="gap: var(--wa-space-m)">
			<SelectAvatar defaultValue={info.avatar} bind:value={avatar}
			></SelectAvatar>

			<wa-input
				defaultValue={info.name}
				oninput={(e: InputEvent) => {
					name = (e.target as WaInput).value!;
				}}
			>
			</wa-input>

			<wa-textarea
				resize="auto"
				rows="2"
				defaultValue={info?.description}
				oninput={(e: InputEvent) => {
					description = (e.target as WaTextarea).value!;
				}}
			>
			</wa-textarea>
		</div>
	</wa-card>
{/await}

<style>
</style>
