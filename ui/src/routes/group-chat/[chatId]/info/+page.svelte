<script lang="ts">
	import '@awesome.me/webawesome/dist/components/button/button.js';
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/dialog/dialog.js';
	import '@awesome.me/webawesome/dist/components/drawer/drawer.js';
	import '@awesome.me/webawesome/dist/components/tag/tag.js';
	import '@awesome.me/webawesome/dist/components/card/card.js';
	import '@awesome.me/webawesome/dist/components/avatar/avatar.js';
	import { m } from '$lib/paraglide/messages.js';

	import { useReactivePromise } from '../../../../stores/use-signal';
	import { getContext } from 'svelte';
	import type { ContactsStore, ChatsStore, PublicKey } from 'dash-chat-stores';
	import { wrapPathInSvg } from '@darksoil-studio/holochain-elements';
	import {
		mdiAccountGroup,
		mdiArrowLeft,
		mdiClose,
		mdiDelete,
		mdiExport,
		mdiKeyVariant,
		mdiPencil,
		mdiPlus,
		mdiPlusCircleOutline,
		mdiPlusOutline,
		mdiSend,
	} from '@mdi/js';
	import Avatar from '../../../../components/Avatar.svelte';
	import WaButton from '@awesome.me/webawesome/dist/components/button/button.js';
	import WaDialog from '@awesome.me/webawesome/dist/components/dialog/dialog.js';

	const chatId = window.location.href.split('/').reverse()[1];

	const chatsStore: ChatsStore = getContext('chats-store');
	const groupChatStore = chatsStore.groupChats(chatId);

	const info = useReactivePromise(groupChatStore.info);
	const members = useReactivePromise(groupChatStore.allMembers);
	const me = useReactivePromise(groupChatStore.me);
</script>

<div class="top-bar">
	<wa-button class="circle" appearance="plain" href="/">
		<wa-icon src={wrapPathInSvg(mdiArrowLeft)}> </wa-icon>
	</wa-button>
	<span class="title" style="flex: 1"></span>
</div>

{#await $me then me}
	<div class="column center-in-desktop" style="gap: var(--wa-space-l)">
		{#await $info then info}
			<div class="column" style="align-items: center">
				<div class="column" style="gap: var(--wa-space-s); align-items: center">
					<wa-avatar image={info.avatar}>
						<wa-icon src={wrapPathInSvg(mdiAccountGroup)}> </wa-icon>
					</wa-avatar>

					<div class="row" style="align-items: center; gap: var(--wa-space-s)">
						<wa-button class="circle" appearance="plain" style="opacity: 0">
						</wa-button>

						<span class="title">{info.name} </span>

						<wa-button
							class="circle"
							appearance="plain"
							href={`/group-chat/${chatId}/info/edit`}
						>
							<wa-icon src={wrapPathInSvg(mdiPencil)}> </wa-icon>
						</wa-button>
					</div>

					<span>{info.description}</span>
				</div>
			</div>
		{/await}

		{#await $members then members}
			<wa-card>
				<div class="column" style="gap: var(--wa-space-m)">
					{m.membersCount({ count: Object.keys(members).length })}
					{#if me.admin}
						<wa-button class="fill member-button" appearance="plain">
							<wa-icon
								auto-width
								style="font-size: 3rem"
								slot="start"
								src={wrapPathInSvg(mdiPlusCircleOutline)}
							></wa-icon>

							{m.addMembers()}
						</wa-button>
					{/if}
					{#each Object.entries(members) as [actorId, member]}
						<wa-button
							data-drawer={`open drawer-${actorId}`}
							class="fill member-button"
							appearance="plain"
						>
							<wa-avatar
								slot="start"
								image={member.profile?.avatar}
								initials={member.profile?.name.slice(0, 2)}
							>
							</wa-avatar>
							<span>{member.profile?.name}</span>

							{#if member.admin}
								<wa-tag slot="end">{m.administrator()}</wa-tag>
							{/if}
						</wa-button>

						<wa-drawer
							light-dismiss
							placement="bottom"
							id={`drawer-${actorId}`}
						>
							<div
								class="column center-in-desktop"
								style="gap: var(--wa-space-m)"
							>
								<div
									class="column"
									style="align-items: center; gap: var(--wa-space-m)"
								>
									<wa-avatar
										image={member.profile?.avatar}
										initials={member.profile?.name.slice(0, 2)}
									>
									</wa-avatar>
									<span>{member.profile?.name}</span>
								</div>

								{#if me.admin && me.actorId !== member.actorId}
									{#if member.admin}
										<wa-button
											class="fill"
											appearance="plain"
											data-dialog={`open demote-dialog-${member.actorId}`}
										>
											<wa-icon slot="start" src={wrapPathInSvg(mdiKeyVariant)}
											></wa-icon>
											{m.demoteFromAdministrator()}
										</wa-button>

										<wa-dialog
											label="Demote from administrator"
											id={`demote-dialog-${member.actorId}`}
										>
											<span
												>{m.areYouSureDemote()}
											</span>
											<wa-button
												variant="neutral"
												appearance="outlined"
												data-dialog="close"
												slot="footer"
												>{m.cancel()}
											</wa-button>
											<wa-button
												slot="footer"
												variant="danger"
												onclick={async (e: CustomEvent) => {
													const button = e.target as WaButton;
													button.loading = true;

													try {
														await groupChatStore.client.demoteFromAdministrator(
															chatId,
															member.actorId,
														);
														const dialog = document.getElementById(
															`demote-dialog-${member.actorId}`,
														) as WaDialog;
														dialog.open = false;
													} catch (e) {}

													button.loading = false;
												}}
											>
												{m.demote()}
											</wa-button>
										</wa-dialog>
									{:else}
										<wa-button
											class="fill"
											appearance="plain"
											data-dialog={`open promote-dialog-${member.actorId}`}
										>
											<wa-icon slot="start" src={wrapPathInSvg(mdiKeyVariant)}
											></wa-icon>
											{m.promoteToAdministrator()}
										</wa-button>

										<wa-dialog
											label="Promote to administrator"
											id={`promote-dialog-${member.actorId}`}
										>
											<span
												>{m.areYouSurePromote()}
											</span>
											<wa-button
												variant="neutral"
												appearance="outlined"
												data-dialog="close"
												slot="footer"
												>{m.cancel()}
											</wa-button>
											<wa-button
												slot="footer"
												variant="brand"
												onclick={async (e: CustomEvent) => {
													const button = e.target as WaButton;
													button.loading = true;

													try {
														await groupChatStore.client.promoteToAdministrator(
															chatId,
															member.actorId,
														);
														const dialog = document.getElementById(
															`promote-dialog-${member.actorId}`,
														) as WaDialog;
														dialog.open = false;
													} catch (e) {}

													button.loading = false;
												}}
											>
												{m.promote()}
											</wa-button>
										</wa-dialog>
									{/if}

									<wa-button
										class="fill"
										appearance="plain"
										data-dialog={`open remove-dialog-${member.actorId}`}
									>
										<wa-icon slot="start" src={wrapPathInSvg(mdiDelete)}
										></wa-icon>
										{m.removeMember()}
									</wa-button>
									<wa-dialog
										label="Remove member"
										id={`remove-dialog-${member.actorId}`}
									>
										<span>{m.areYouSureRemoveMember()}</span>
										<wa-button
											variant="neutral"
											appearance="outlined"
											data-dialog="close"
											slot="footer"
											>{m.cancel()}
										</wa-button>
										<wa-button
											slot="footer"
											variant="danger"
											onclick={async (e: CustomEvent) => {
												const button = e.target as WaButton;
												button.loading = true;

												try {
													await groupChatStore.client.removeMember(
														chatId,
														member.actorId,
													);
													const dialog = document.getElementById(
														`remove-dialog-${member.actorId}`,
													) as WaDialog;
													dialog.open = false;
												} catch (e) {}

												button.loading = false;
											}}
										>
											{m.removeMember()}
										</wa-button>
									</wa-dialog>
								{/if}
							</div>
						</wa-drawer>
					{/each}
				</div>
			</wa-card>
		{/await}

		<div class="column" style="gap: var(--wa-space-m)">
			<wa-button
				class="fill"
				variant="danger"
				appearance="plain"
				data-dialog="open leave-group-dialog"
			>
				<wa-icon src={wrapPathInSvg(mdiExport)} slot="start"></wa-icon>

				{m.leaveGroup()}
			</wa-button>
			<wa-dialog label="Leave group" id="leave-group-dialog">
				<span>{m.areYouSureLeaveGroup()}</span>
				<wa-button
					variant="neutral"
					appearance="outlined"
					data-dialog="close"
					slot="footer"
					>{m.cancel()}
				</wa-button>
				<wa-button
					slot="footer"
					variant="danger"
					onclick={async (e: CustomEvent) => {
						const button = e.target as WaButton;
						button.loading = true;

						try {
							await groupChatStore.client.leaveGroup();
							const dialog = document.getElementById(
								'leave-group-dialog',
							) as WaDialog;
							dialog.open = false;
						} catch (e) {}

						button.loading = false;
					}}
				>
				{m.leaveGroup()}
				</wa-button>
			</wa-dialog>

			<wa-button
				class="fill"
				variant="danger"
				appearance="plain"
				data-dialog="open delete-group-dialog"
			>
				<wa-icon src={wrapPathInSvg(mdiClose)} slot="start"></wa-icon>

				{m.deleteGroup()}
			</wa-button>
			<wa-dialog label="Delete group" id="delete-group-dialog">
				<span>{m.areYouSureDeleteGroup()}</span>
				<wa-button
					variant="neutral"
					appearance="outlined"
					data-dialog="close"
					slot="footer"
					>{m.cancel()}
				</wa-button>
				<wa-button
					slot="footer"
					variant="danger"
					onclick={async (e: CustomEvent) => {
						const button = e.target as WaButton;
						button.loading = true;

						try {
							await groupChatStore.client.deleteGroup();
							const dialog = document.getElementById(
								'delete-group-dialog',
							) as WaDialog;
							dialog.open = false;
						} catch (e) {}

						button.loading = false;
					}}
				>
					{m.deleteGroup()}
				</wa-button>
			</wa-dialog>
		</div>
	</div>
{/await}

<style>
	wa-button.member-button::part(base) {
		height: 68px;
	}
</style>
