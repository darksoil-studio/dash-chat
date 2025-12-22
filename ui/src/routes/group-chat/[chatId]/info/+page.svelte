<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/avatar/avatar.js';
	import { m } from '$lib/paraglide/messages.js';

	import { useReactivePromise } from '../../../../stores/use-signal';
	import { getContext } from 'svelte';
	import type { ContactsStore, ChatsStore, PublicKey } from 'dash-chat-stores';
	import { wrapPathInSvg } from '@darksoil-studio/holochain-elements';
	import {
		mdiAccountGroup,
		mdiClose,
		mdiDelete,
		mdiExport,
		mdiKeyVariant,
		mdiPencil,
		mdiPlusCircle,
	} from '@mdi/js';
	import {
		Page,
		Navbar,
		NavbarBackLink,
		Button,
		Card,
		Link,
		List,
		ListItem,
		Chip,
		Dialog,
		DialogButton,
		Sheet,
		ActionsButton,
		BlockTitle,
	} from 'konsta/svelte';
	import Layout from '../../../+layout.svelte';

	const chatId = window.location.href.split('/').reverse()[1];

	const chatsStore: ChatsStore = getContext('chats-store');
	const groupChatStore = chatsStore.groupChats(chatId);

	const info = useReactivePromise(groupChatStore.info);
	const members = useReactivePromise(groupChatStore.allMembers);
	const me = useReactivePromise(groupChatStore.me);

	let sheetOpenFor = $state<string | null>(null);
	let dialogType = $state<
		'demote' | 'promote' | 'remove' | 'leave' | 'delete' | null
	>(null);
	let dialogActorId = $state<string | null>(null);
	let loading = $state(false);

	async function handleDemote(actorId: string) {
		loading = true;
		try {
			await groupChatStore.client.demoteFromAdministrator(chatId, actorId);
			dialogType = null;
			dialogActorId = null;
		} catch (e) {
			console.error(e);
		}
		loading = false;
	}

	async function handlePromote(actorId: string) {
		loading = true;
		try {
			await groupChatStore.client.promoteToAdministrator(chatId, actorId);
			dialogType = null;
			dialogActorId = null;
		} catch (e) {
			console.error(e);
		}
		loading = false;
	}

	async function handleRemove(actorId: string) {
		loading = true;
		try {
			await groupChatStore.client.removeMember(chatId, actorId);
			dialogType = null;
			dialogActorId = null;
		} catch (e) {
			console.error(e);
		}
		loading = false;
	}

	async function handleLeaveGroup() {
		loading = true;
		try {
			await groupChatStore.client.leaveGroup();
			dialogType = null;
			window.location.href = '/';
		} catch (e) {
			console.error(e);
		}
		loading = false;
	}

	async function handleDeleteGroup() {
		loading = true;
		try {
			await groupChatStore.client.deleteGroup();
			dialogType = null;
			window.location.href = '/';
		} catch (e) {
			console.error(e);
		}
		loading = false;
	}
</script>

<Page>
	<Navbar title="" transparent={true}>
		{#snippet left()}
			<NavbarBackLink onClick={() => (window.location.href = '/')} />
		{/snippet}

		{#snippet right()}
			<Link href={`/group-chat/${chatId}/info/edit`} iconOnly>
				<wa-icon src={wrapPathInSvg(mdiPencil)}> </wa-icon>
			</Link>
		{/snippet}
	</Navbar>

	{#await $me then me}
		<div class="column p-2" style="flex: 1">
			<div class="column center-in-desktop gap-8 p-2">
				{#await $info then info}
					<div class="column" style="align-items: center; gap: 1rem">
						<wa-avatar image={info.avatar} style="--size: 5rem">
							<wa-icon src={wrapPathInSvg(mdiAccountGroup)}> </wa-icon>
						</wa-avatar>

						<span class="text-xl font-semibold">{info.name}</span>

						<span class="quiet">{info.description}</span>
					</div>
				{/await}

				{#await $members then members}
					<BlockTitle>
						{m.membersCount({ count: Object.keys(members).length })}</BlockTitle
					>
					<List nested>
						{#if me.admin}
							<ListItem title={m.addMembers()} link chevron={false}>
								{#snippet media()}
									<wa-icon
										style="font-size: 2.5rem;"
										src={wrapPathInSvg(mdiPlusCircle)}
									></wa-icon>
								{/snippet}
							</ListItem>
						{/if}

						{#each Object.entries(members) as [actorId, member]}
							<ListItem
								link
								chevron={false}
								title={member.profile?.name}
								onclick={() => (sheetOpenFor = actorId)}
							>
								{#snippet media()}
									<wa-avatar
										image={member.profile?.avatar}
										initials={member.profile?.name.slice(0, 2)}
									></wa-avatar>
								{/snippet}

								{#snippet after()}
									{#if member.admin}
										<Chip>{m.administrator()}</Chip>
									{/if}
								{/snippet}
							</ListItem>

							{#if sheetOpenFor === actorId}
								<Sheet
									class="pb-safe"
									opened={sheetOpenFor === actorId}
									onBackdropClick={() => (sheetOpenFor = null)}
								>
									<div
										class="flex-col gap-4 py-4"
										style="display: flex; align-items: center;"
									>
										<wa-avatar
											image={member.profile?.avatar}
											initials={member.profile?.name.slice(0, 2)}
										></wa-avatar>
										<span class="font-semibold">{member.profile?.name}</span>
									</div>

									<List nested class="mb-2">
										{#if me.admin}
											{#if member.admin}
												<ListItem
													link
													title={m.demoteFromAdministrator()}
													onClick={() => {
														dialogType = 'demote';
														dialogActorId = actorId;
														sheetOpenFor = null;
													}}
												>
													{#snippet media()}
														<wa-icon src={wrapPathInSvg(mdiKeyVariant)}
														></wa-icon>
													{/snippet}
												</ListItem>
											{:else}
												<ListItem
													link
													title={m.promoteToAdministrator()}
													onClick={() => {
														dialogType = 'promote';
														dialogActorId = actorId;
														sheetOpenFor = null;
													}}
												>
													{#snippet media()}
														<wa-icon src={wrapPathInSvg(mdiKeyVariant)}
														></wa-icon>
													{/snippet}
												</ListItem>
											{/if}

											<ListItem
												link
												title={m.removeMember()}
												onClick={() => {
													dialogType = 'remove';
													dialogActorId = actorId;
													sheetOpenFor = null;
												}}
											>
												{#snippet media()}
													<wa-icon src={wrapPathInSvg(mdiDelete)}></wa-icon>
												{/snippet}
											</ListItem>
										{/if}
									</List>
								</Sheet>
							{/if}
						{/each}
					</List>

					<List nested class="z-1">
						<ListItem
							title={m.leaveGroup()}
							link
							chevron={false}
							onClick={() => (dialogType = 'leave')}
							colors={{
								primaryTextIos: 'text-red-500',
								primaryTextMaterial: 'text-red-600',
							}}
						>
							{#snippet media()}
								<wa-icon class="big" src={wrapPathInSvg(mdiExport)}></wa-icon>
							{/snippet}
						</ListItem>

						<ListItem
							title={m.deleteGroup()}
							link
							chevron={false}
							onClick={() => (dialogType = 'delete')}
							colors={{
								primaryTextIos: 'text-red-500',
								primaryTextMaterial: 'text-red-600',
							}}
						>
							{#snippet media()}
								<wa-icon class="big" src={wrapPathInSvg(mdiClose)}></wa-icon>
							{/snippet}
						</ListItem>
					</List>
				{/await}
			</div>
		</div>
	{/await}

	<!-- Dialogs -->
	<Dialog
		opened={dialogType === 'demote' && dialogActorId !== null}
		onBackdropClick={() => {
			dialogType = null;
			dialogActorId = null;
		}}
	>
		{#snippet title()}
			{m.demoteFromAdministrator()}
		{/snippet}
		<span>{m.areYouSureDemote()}</span>
		{#snippet buttons()}
			<DialogButton
				onClick={() => {
					dialogType = null;
					dialogActorId = null;
				}}
			>
				{m.cancel()}
			</DialogButton>
			<DialogButton
				strong
				onClick={() => dialogActorId && handleDemote(dialogActorId)}
				disabled={loading}
			>
				{loading ? '...' : m.demote()}
			</DialogButton>
		{/snippet}
	</Dialog>

	<Dialog
		opened={dialogType === 'promote' && dialogActorId !== null}
		onBackdropClick={() => {
			dialogType = null;
			dialogActorId = null;
		}}
	>
		{#snippet title()}
			{m.promoteToAdministrator()}
		{/snippet}
		<span>{m.areYouSurePromote()}</span>
		{#snippet buttons()}
			<DialogButton
				onClick={() => {
					dialogType = null;
					dialogActorId = null;
				}}
			>
				{m.cancel()}
			</DialogButton>
			<DialogButton
				onClick={() => dialogActorId && handlePromote(dialogActorId)}
				disabled={loading}
				strong
			>
				{loading ? '...' : m.promote()}
			</DialogButton>
		{/snippet}
	</Dialog>

	<Dialog
		opened={dialogType === 'remove' && dialogActorId !== null}
		onBackdropClick={() => {
			dialogType = null;
			dialogActorId = null;
		}}
	>
		{#snippet title()}
			{m.removeMember()}
		{/snippet}
		<span>{m.areYouSureRemoveMember()}</span>
		{#snippet buttons()}
			<DialogButton
				onClick={() => {
					dialogType = null;
					dialogActorId = null;
				}}
			>
				{m.cancel()}
			</DialogButton>
			<DialogButton
				strong
				onClick={() => dialogActorId && handleRemove(dialogActorId)}
				disabled={loading}
			>
				{loading ? '...' : m.remove()}
			</DialogButton>
		{/snippet}
	</Dialog>

	<Dialog
		opened={dialogType === 'leave'}
		onBackdropClick={() => (dialogType = null)}
	>
		{#snippet title()}
			{m.leaveGroup()}
		{/snippet}
		<span>{m.areYouSureLeaveGroup()}</span>
		{#snippet buttons()}
			<DialogButton onClick={() => (dialogType = null)}
				>{m.cancel()}</DialogButton
			>
			<DialogButton strong onClick={handleLeaveGroup} disabled={loading}>
				{loading ? '...' : m.leave()}
			</DialogButton>
		{/snippet}
	</Dialog>

	<Dialog
		opened={dialogType === 'delete'}
		onBackdropClick={() => (dialogType = null)}
	>
		{#snippet title()}
			{m.deleteGroup()}
		{/snippet}
		<span>{m.areYouSureDeleteGroup()}</span>
		{#snippet buttons()}
			<DialogButton onClick={() => (dialogType = null)}
				>{m.cancel()}</DialogButton
			>
			<DialogButton strong onClick={handleDeleteGroup} disabled={loading}>
				{loading ? '...' : m.delete()}
			</DialogButton>
		{/snippet}
	</Dialog>
</Page>

<style>
	wa-icon.big {
		font-size: 2rem;
	}
</style>
