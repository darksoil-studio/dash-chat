<script lang="ts">
	import '@awesome.me/webawesome/dist/components/textarea/textarea.js';
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/button/button.js';
	import type WaTextarea from '@awesome.me/webawesome/dist/components/textarea/textarea.js';

	import { useReactivePromise } from '../../../stores/use-signal';
	import { getContext } from 'svelte';
	import type { ContactsStore, ChatsStore, PublicKey } from 'dash-chat-stores';
	import { wrapPathInSvg } from '@darksoil-studio/holochain-elements';
	import { mdiArrowLeft, mdiSend } from '@mdi/js';

	const chatId = window.location.href.split('/').reverse()[0];

	const chatsStore: ChatsStore = getContext('chats-store');
	const store = chatsStore.groupChats(chatId);

	const messages = useReactivePromise(store.messages);
	let textarea: WaTextarea;

	async function sendMessage() {
		const message = textarea.value;
		if (!message || message.trim() === '') return;

		await store.sendMessage(message);
		textarea.value = '';
	}
</script>

<div class="top-bar">
	<wa-button
		class="circle"
		appearance="plain"
		onclick={() => window.history.back()}
	>
			<wa-icon src={wrapPathInSvg(mdiArrowLeft)}> </wa-icon>
	</wa-button>
	<span class="title" style="flex: 1"></span>
</div>

<div class="column center-in-desktop" style="gap: var(--wa-space-l)">
	{#await $messages then messages}
		{#each messages as message}
			hey{message}
		{/each}
	{/await}

	<div class="row" style="align-items: center; gap: var(--wa-space-s)">
		<wa-textarea resize="auto" rows="1" bind:this={textarea} style="flex: 1"> </wa-textarea>

		<wa-button class="circle" appearance="plain" onclick={sendMessage}>
			<wa-icon src={wrapPathInSvg(mdiSend)}> </wa-icon>
		</wa-button>
	</div>
</div>

<style>
</style>
