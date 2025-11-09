<script lang="ts">
	import '@awesome.me/webawesome/dist/components/spinner/spinner.js';
	import type { UsersStore } from 'dash-chat-stores';
	import { getContext } from 'svelte';
	import { useReactivePromise } from '../../stores/use-signal';

	const usersStore: UsersStore = getContext('users-store');

	const me = useReactivePromise(usersStore.me);
</script>

<div class="top-bar">
	<wa-button class="circle" href="/" appearance="plain">
		<wa-icon name="arrow-left"> </wa-icon>
	</wa-button>

	<span class="title">My profile </span>

	<div style="flex: 1"></div>
	<wa-button href="/my-profile/edit" appearance="plain">
		<wa-icon slot="start" name="pencil"> </wa-icon>
		Edit
	</wa-button>
</div>

{#await $me}
	<wa-spinner> </wa-spinner>
{:then me}
	<wa-card class="center-in-desktop" style="margin: var(--wa-space-m);">
		<div class="row" style="align-items: center; gap: var(--wa-space-s)">
			<wa-avatar
				image={me?.profile?.avatar}
				initials={me?.profile?.name.slice(0, 2)}
			>
			</wa-avatar>
			{me?.profile?.name}
		</div>
	</wa-card>
{/await}
