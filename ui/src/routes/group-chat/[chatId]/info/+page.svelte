<script lang="ts">
	import '@awesome.me/webawesome/dist/components/button/button.js';
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/avatar/avatar.js';

	import { useReactivePromise } from '../../../../stores/use-signal';
	import { getContext } from 'svelte';
	import type { ContactsStore, ChatsStore, PublicKey } from 'dash-chat-stores';
	import { wrapPathInSvg } from '@darksoil-studio/holochain-elements';
	import { mdiAccountGroup, mdiArrowLeft, mdiPencil, mdiSend } from '@mdi/js';

	const chatId = window.location.href.split('/').reverse()[1];

	const chatsStore: ChatsStore = getContext('chats-store');
	const store = chatsStore.groupChats(chatId);
	const info = useReactivePromise(store.info);
</script>

<div class="top-bar">
	<wa-button
		class="circle"
		appearance="plain"
		href="/"
	>
		<wa-icon src={wrapPathInSvg(mdiArrowLeft)}> </wa-icon>
	</wa-button>
	<span class="title" style="flex: 1"></span>
</div>

{#await $info then info}
	<div class="column" style="align-items: center">
		<div class="column" style="gap: var(--wa-space-s); align-items: center">
			<wa-avatar image={info.avatar}>
				<wa-icon src={wrapPathInSvg(mdiAccountGroup)}> </wa-icon>
			</wa-avatar>

			<div class="row" style="align-items: center; gap: var(--wa-space-s)">

				<wa-button
					class="circle"
					appearance="plain"
					style="opacity: 0"
				>
				</wa-button>

				<span class="title">{info.name} </span>

				<wa-button
					class="circle"
					appearance="plain"
					href={`/group-chat/${chatId}/info/edit`}
				>
					<wa-icon src={wrapPathInSvg(mdiPencil)}> </wa-icon>
				</wa-button>
			</div>

			<span>{info.description} </span>
		</div>
	</div>
{/await}

<style>
</style>
