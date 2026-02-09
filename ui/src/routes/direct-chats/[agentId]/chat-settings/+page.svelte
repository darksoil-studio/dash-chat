<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/avatar/avatar.js';
	import { fullName, type ChatsStore } from 'dash-chat-stores';
	import { getContext } from 'svelte';
	import { goto } from '$app/navigation';
	import { useReactivePromise } from '$lib/stores/use-signal';
	import {
		mdiBellOutline,
		mdiMagnify,
		mdiPalette,
		mdiPlusCircle,
		mdiChevronRight,
	} from '@mdi/js';
	import { wrapPathInSvg } from '$lib/utils/icon';
	import { showToast } from '$lib/utils/toasts';
	import { m } from '$lib/paraglide/messages.js';
	import {
		List,
		ListItem,
		Navbar,
		NavbarBackLink,
		Page,
		Preloader,
	} from 'konsta/svelte';
	import { page } from '$app/state';
	let agentId = page.params.agentId!;

	const chatsStore: ChatsStore = getContext('chats-store');
	const store = chatsStore.directChats(agentId);

	const peerProfile = useReactivePromise(store.peerProfile);

	function comingSoon() {
		showToast(m.comingSoon());
	}
</script>

<Page>
	{#await $peerProfile}
		<div
			class="column"
			style="height: 100%; align-items: center; justify-content: center"
		>
			<Preloader />
		</div>
	{:then profile}
		<Navbar transparent={true}>
			{#snippet left()}
				<NavbarBackLink onClick={() => goto(`/direct-chats/${agentId}`)} />
			{/snippet}
		</Navbar>

		<div class="column" style="flex: 1">
			<div class="column center-in-desktop">
				<div class="column m-6 gap-2" style="align-items: center">
					<wa-avatar
						image={profile?.avatar}
						initials={profile?.name.slice(0, 2)}
						style="--size: 80px;"
					>
					</wa-avatar>

					<button
						class="flex items-center gap-1"
						onclick={comingSoon}
					>
						<span class="text-xl font-semibold"
							>{fullName(profile!)}</span
						>
						<wa-icon
							class="small-icon quiet"
							src={wrapPathInSvg(mdiChevronRight)}
						></wa-icon>
					</button>

					{#if profile?.about}
						<span class="quiet text-center">{profile.about}</span>
					{/if}
				</div>

				<div class="my-4 flex justify-center gap-4">
					<button
						class="flex w-[70px] flex-col items-center gap-1 rounded-xl bg-blue-50 p-3 dark:bg-blue-900/30"
						onclick={comingSoon}
					>
						<wa-icon src={wrapPathInSvg(mdiBellOutline)}></wa-icon>
						<span class="text-xs">{m.mute()}</span>
					</button>
					<button
						class="flex w-[70px] flex-col items-center gap-1 rounded-xl bg-blue-50 p-3 dark:bg-blue-900/30"
						onclick={comingSoon}
					>
						<wa-icon src={wrapPathInSvg(mdiMagnify)}></wa-icon>
						<span class="text-xs">{m.search()}</span>
					</button>
				</div>

				<List nested strongIos insetIos>
					<ListItem
						link
						chevron={false}
						title={m.chatColorAndWallpaper()}
						onClick={comingSoon}
					>
						{#snippet media()}
							<wa-icon
								style="font-size: 2rem;"
								src={wrapPathInSvg(mdiPalette)}
							></wa-icon>
						{/snippet}
					</ListItem>
				</List>

				<div class="mt-6 px-4">
					<span class="font-bold">{m.noGroupsInCommonTitle()}</span>
				</div>

				<List nested strongIos insetIos>
					<ListItem
						link
						chevron={false}
						title={m.addToAGroup()}
						onClick={comingSoon}
					>
						{#snippet media()}
							<wa-icon
								style="font-size: 2rem;"
								src={wrapPathInSvg(mdiPlusCircle)}
							></wa-icon>
						{/snippet}
					</ListItem>
				</List>
			</div>
		</div>
	{/await}
</Page>
