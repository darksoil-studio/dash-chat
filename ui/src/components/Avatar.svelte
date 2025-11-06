<script lang="ts">
	import '@awesome.me/webawesome/dist/components/skeleton/skeleton.js';
	import '@awesome.me/webawesome/dist/components/avatar/avatar.js';
	import type { UsersStore, UserId } from 'dash-chat-stores';
	import { getContext } from 'svelte';
	import { useSignal } from '../stores/use-signal';

	let { userId }: { userId: UserId } = $props();

	const usersStore: UsersStore = getContext('users-store');

	const user = useSignal(usersStore.users, userId);
</script>

{#await $user}
	<wa-skeleton> </wa-skeleton>
{:then user}
	<wa-avatar image={user.profile?.avatar}> </wa-avatar>
{/await}
