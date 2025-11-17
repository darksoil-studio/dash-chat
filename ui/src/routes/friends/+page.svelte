<script lang="ts">
	import '@awesome.me/webawesome/dist/components/spinner/spinner.js';
	import type { UsersStore } from 'dash-chat-stores';
	import { getContext } from 'svelte';
	import { useReactivePromise } from '../../stores/use-signal';
	import AddFriend from '../../friends/AddFriend.svelte';
	import Avatar from '../../components/Avatar.svelte';

	const usersStore: UsersStore = getContext('users-store');

	const me = useReactivePromise(usersStore.me);

	const friends = ['alice', 'bobbi'];
</script>

<div class="top-bar">
	<wa-button class="circle" href="/" appearance="plain">
		<wa-icon name="arrow-left"> </wa-icon>
	</wa-button>

	<span class="title">Friends</span>
</div>

{#each friends as friend}
	<div class="row" style="margin: var(--wa-space-xs)">
		<Avatar userId={friend}></Avatar>
		{friend}
	</div>
{/each}

<div class="row" style="margin: var(--wa-space-xs)">
	<AddFriend></AddFriend>
</div>
