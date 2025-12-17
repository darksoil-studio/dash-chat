<script lang="ts">
	import '@awesome.me/webawesome/dist/components/skeleton/skeleton.js';
	import '@awesome.me/webawesome/dist/components/avatar/avatar.js';
	import type { ContactsStore, PublicKey } from 'dash-chat-stores';
	import { getContext } from 'svelte';
	import { useReactivePromise } from '../stores/use-signal';

	let { chatActorId }: { chatActorId: PublicKey} = $props();

	const contactsStore: ContactsStore = getContext('contacts-store');

	const profile= useReactivePromise(contactsStore.profiles, chatActorId);
</script>

{#await $profile}
	<wa-skeleton> </wa-skeleton>
{:then profile}
	<wa-avatar
		image={profile?.avatar}
		initials={profile?.name.slice(0, 2)}
	>
	</wa-avatar>
{/await}
