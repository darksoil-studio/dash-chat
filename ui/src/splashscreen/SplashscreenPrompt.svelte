<script lang="ts">
	import '@awesome.me/webawesome/dist/components/spinner/spinner.js'
	import type { UsersStore } from 'dash-chat-stores';
	import { getContext } from 'svelte';
	import { useSignal } from '../stores/use-signal';
	import CreateProfile from '../profiles/CreateProfile.svelte';

	const usersStore: UsersStore = getContext('users-store');

	const me = useSignal(usersStore.me);
</script>

{#await $me}
	<wa-spinner> </wa-spinner>
{:then me}
	{#if me?.profile}
		<slot></slot>
	{:else}
		<div class="column" style="flex: 1; align-items: center; justify-content: center">
			<CreateProfile></CreateProfile>
		</div>
	{/if}
{/await}
