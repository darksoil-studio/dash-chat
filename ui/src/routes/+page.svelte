<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/button/button.js';
	import type { UsersStore } from 'dash-chat-stores';
	import { getContext } from 'svelte';
	import { useReactivePromise } from '../stores/use-signal';
	import Avatar from '../components/Avatar.svelte';

	const usersStore: UsersStore = getContext('users-store');

	const myPubKey = useReactivePromise (usersStore.myPubKey);
</script>

{#await $myPubKey then myPubKey}
	<div class="column" style="flex: 1">
		<div class="top-bar">
			<a href="/my-profile">
				<Avatar userId={myPubKey}></Avatar>
			</a>
			<div style="flex: 1">
				</div>
				<wa-button class="circle" href="/friends" appearance="plain">
					<wa-icon name="user-group">
					</wa-icon>
				</wa-button>
		</div>
	</div>
{/await}

<style>

</style>
