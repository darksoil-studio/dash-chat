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
		mdiPlusCircleOutline,
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
		Actions,
		ActionsGroup,
		ActionsButton,
		ActionsLabel,
	} from 'konsta/svelte';

	const chatId = window.location.href.split('/').reverse()[1];

	const chatsStore: ChatsStore = getContext('chats-store');
	const groupChatStore = chatsStore.groupChats(chatId);

	const info = useReactivePromise(groupChatStore.info);
	const members = useReactivePromise(groupChatStore.allMembers);
	const me = useReactivePromise(groupChatStore.me);

	let actionsOpenFor = $state<string | null>(null);
	let dialogType = $state<'demote' | 'promote' | 'remove' | 'leave' | 'delete' | null>(null);
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
	<Navbar title="">
		{#snippet left()}
			<NavbarBackLink onClick={() => (window.location.href = '/')} />
		{/snippet}
	</Navbar>

	{#await $me then me}
		<div class="column center-in-desktop" style="gap: 2rem; padding: 1rem">
			{#await $info then info}
				<div class="column" style="align-items: center; gap: 1rem">
					<wa-avatar image={info.avatar} style="--size: 5rem">
						<wa-icon src={wrapPathInSvg(mdiAccountGroup)}> </wa-icon>
					</wa-avatar>

					<div class="row" style="align-items: center; gap: 0.5rem">
						<span class="text-xl font-semibold">{info.name}</span>
						<Link href={`/group-chat/${chatId}/info/edit`} iconOnly>
							<wa-icon src={wrapPathInSvg(mdiPencil)}> </wa-icon>
						</Link>
					</div>

					<span class="quiet">{info.description}</span>
				</div>
			{/await}

			{#await $members then members}
				<Card>
					<div class="column">
						<span class="font-semibold">
							{m.membersCount({ count: Object.keys(members).length })}
						</span>

						{#if me.admin}
							<Button clear class="w-full justify-start gap-3" style="height: 68px">
								<wa-icon
									style="font-size: 3rem"
									src={wrapPathInSvg(mdiPlusCircleOutline)}
								></wa-icon>
								{m.addMembers()}
							</Button>
						{/if}

						{#each Object.entries(members) as [actorId, member]}
							<button
								class="gap-3 p-3 w-full text-left hover:bg-gray-50 rounded"
								style="display: flex; align-items: center; min-height: 68px"
								onclick={() => (actionsOpenFor = actorId)}
							>
								<wa-avatar
									image={member.profile?.avatar}
									initials={member.profile?.name.slice(0, 2)}
								></wa-avatar>
								<span class="flex-1">{member.profile?.name}</span>
								{#if member.admin}
									<Chip>{m.administrator()}</Chip>
								{/if}
							</button>

							{#if actionsOpenFor === actorId}
								<Actions
									opened={actionsOpenFor === actorId}
									onBackdropClick={() => (actionsOpenFor = null)}
								>
									<ActionsGroup>
										<ActionsLabel>
											<div class="flex-col gap-2 py-2" style="display: flex; align-items: center;">
												<wa-avatar
													image={member.profile?.avatar}
													initials={member.profile?.name.slice(0, 2)}
												></wa-avatar>
												<span>{member.profile?.name}</span>
											</div>
										</ActionsLabel>

										{#if me.admin && me.actorId !== member.actorId}
											{#if member.admin}
												<ActionsButton
													onClick={() => {
														dialogType = 'demote';
														dialogActorId = actorId;
														actionsOpenFor = null;
													}}
												>
													<wa-icon src={wrapPathInSvg(mdiKeyVariant)}></wa-icon>
													{m.demoteFromAdministrator()}
												</ActionsButton>
											{:else}
												<ActionsButton
													onClick={() => {
														dialogType = 'promote';
														dialogActorId = actorId;
														actionsOpenFor = null;
													}}
												>
													<wa-icon src={wrapPathInSvg(mdiKeyVariant)}></wa-icon>
													{m.promoteToAdministrator()}
												</ActionsButton>
											{/if}

											<ActionsButton
												colors={{ textIos: 'text-red-500', textMaterial: 'text-red-600' }}
												onClick={() => {
													dialogType = 'remove';
													dialogActorId = actorId;
													actionsOpenFor = null;
												}}
											>
												<wa-icon src={wrapPathInSvg(mdiDelete)}></wa-icon>
												{m.removeMember()}
											</ActionsButton>
										{/if}
									</ActionsGroup>

									<ActionsGroup>
										<ActionsButton onClick={() => (actionsOpenFor = null)}>
											{m.cancel()}
										</ActionsButton>
									</ActionsGroup>
								</Actions>
							{/if}
						{/each}
					</div>
				</Card>
			{/await}

			<div class="column" style="gap: 1rem">
				<Button
					clear
					class="w-full justify-start gap-3"
					colors={{ textIos: 'text-red-500', textMaterial: 'text-red-600' }}
					onClick={() => (dialogType = 'leave')}
				>
					<wa-icon src={wrapPathInSvg(mdiExport)}></wa-icon>
					{m.leaveGroup()}
				</Button>

				<Button
					clear
					class="w-full justify-start gap-3"
					colors={{ textIos: 'text-red-500', textMaterial: 'text-red-600' }}
					onClick={() => (dialogType = 'delete')}
				>
					<wa-icon src={wrapPathInSvg(mdiClose)}></wa-icon>
					{m.deleteGroup()}
				</Button>
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
		title={m.demoteFromAdministrator()}
	>
		<p class="p-4">{m.areYouSureDemote()}</p>
		{#snippet buttons()}
			<Button
				onClick={() => {
					dialogType = null;
					dialogActorId = null;
				}}
			>
				{m.cancel()}
			</Button>
			<Button
				colors={{ fillBgIos: 'bg-red-500', fillBgMaterial: 'bg-red-600' }}
				onClick={() => dialogActorId && handleDemote(dialogActorId)}
				disabled={loading}
			>
				{loading ? '...' : m.demote()}
			</Button>
		{/snippet}
	</Dialog>

	<Dialog
		opened={dialogType === 'promote' && dialogActorId !== null}
		onBackdropClick={() => {
			dialogType = null;
			dialogActorId = null;
		}}
		title={m.promoteToAdministrator()}
	>
		<p class="p-4">{m.areYouSurePromote()}</p>
		{#snippet buttons()}
			<Button
				onClick={() => {
					dialogType = null;
					dialogActorId = null;
				}}
			>
				{m.cancel()}
			</Button>
			<Button onClick={() => dialogActorId && handlePromote(dialogActorId)} disabled={loading}>
				{loading ? '...' : m.promote()}
			</Button>
		{/snippet}
	</Dialog>

	<Dialog
		opened={dialogType === 'remove' && dialogActorId !== null}
		onBackdropClick={() => {
			dialogType = null;
			dialogActorId = null;
		}}
		title={m.removeMember()}
	>
		<p class="p-4">{m.areYouSureRemoveMember()}</p>
		{#snippet buttons()}
			<Button
				onClick={() => {
					dialogType = null;
					dialogActorId = null;
				}}
			>
				{m.cancel()}
			</Button>
			<Button
				colors={{ fillBgIos: 'bg-red-500', fillBgMaterial: 'bg-red-600' }}
				onClick={() => dialogActorId && handleRemove(dialogActorId)}
				disabled={loading}
			>
				{loading ? '...' : m.removeMember()}
			</Button>
		{/snippet}
	</Dialog>

	<Dialog
		opened={dialogType === 'leave'}
		onBackdropClick={() => (dialogType = null)}
		title={m.leaveGroup()}
	>
		<p class="p-4">{m.areYouSureLeaveGroup()}</p>
		{#snippet buttons()}
			<Button onClick={() => (dialogType = null)}>{m.cancel()}</Button>
			<Button
				colors={{ fillBgIos: 'bg-red-500', fillBgMaterial: 'bg-red-600' }}
				onClick={handleLeaveGroup}
				disabled={loading}
			>
				{loading ? '...' : m.leaveGroup()}
			</Button>
		{/snippet}
	</Dialog>

	<Dialog
		opened={dialogType === 'delete'}
		onBackdropClick={() => (dialogType = null)}
		title={m.deleteGroup()}
	>
		<p class="p-4">{m.areYouSureDeleteGroup()}</p>
		{#snippet buttons()}
			<Button onClick={() => (dialogType = null)}>{m.cancel()}</Button>
			<Button
				colors={{ fillBgIos: 'bg-red-500', fillBgMaterial: 'bg-red-600' }}
				onClick={handleDeleteGroup}
				disabled={loading}
			>
				{loading ? '...' : m.deleteGroup()}
			</Button>
		{/snippet}
	</Dialog>
</Page>
