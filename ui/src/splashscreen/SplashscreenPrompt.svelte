<script lang="ts">
	import '@awesome.me/webawesome/dist/components/spinner/spinner.js'
	import { getContext } from 'svelte';
	import { useReactivePromise} from '../stores/use-signal';
	import CreateProfile from '../profiles/CreateProfile.svelte';
	import type { ContactsStore } from 'dash-chat-stores';

	const contactsStore: ContactsStore = getContext('contacts-store');

	const myProfile = useReactivePromise(contactsStore.myProfile);
</script>

{#await $myProfile }
	<wa-spinner> </wa-spinner>
{:then myProfile }
	{#if myProfile}
		<slot></slot>
	{:else}
		<div class="column" style="flex: 1; align-items: center; justify-content: center">
			<CreateProfile></CreateProfile>
		</div>
	{/if}
{/await}
