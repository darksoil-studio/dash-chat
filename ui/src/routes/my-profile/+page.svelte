<script lang="ts">
	import '@awesome.me/webawesome/dist/components/spinner/spinner.js';
	import type { UsersStore } from 'dash-chat-stores';

	
	import { getContext } from 'svelte';
	import { useSignal } from '../../stores/use-signal';

	const usersStore: UsersStore = getContext('users-store');
	

	const me = useSignal(usersStore.me)
</script>

{#await $me}
	<wa-spinner>
	</wa-spinner>
{:then me}
	{me?.profile?.name}
{/await}
