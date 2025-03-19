import {
	FriendsStore,
	friendsStoreContext,
} from '@darksoil-studio/friends-zome';
import { scanQrCodeAndSendFriendRequest } from '@darksoil-studio/friends-zome/dist/elements/friend-request-qr-code.js';
import {
	MessengerStore,
	messengerStoreContext,
} from '@darksoil-studio/messenger-zome';
import '@darksoil-studio/messenger-zome/dist/elements/all-chats.js';
import '@darksoil-studio/messenger-zome/dist/elements/group-chat.js';
import '@darksoil-studio/messenger-zome/dist/elements/peer-chat.js';
import {
	ProfilesProvider,
	profilesProviderContext,
} from '@darksoil-studio/profiles-provider';
import '@darksoil-studio/profiles-provider/dist/elements/profile-list-item.js';
import {
	AdminWebsocket,
	AppClient,
	decodeHashFromBase64,
	encodeHashToBase64,
} from '@holochain/client';
import { consume } from '@lit/context';
import { msg } from '@lit/localize';
import {
	mdiAccount,
	mdiAccountGroup,
	mdiAccountMultiplePlus,
	mdiAccountPlus,
	mdiArrowLeft,
	mdiChat,
	mdiChatOutline,
	mdiDotsVertical,
	mdiMessagePlus,
} from '@mdi/js';
import { SlButton, SlDialog } from '@shoelace-style/shoelace';
import '@shoelace-style/shoelace/dist/components/badge/badge.js';
import '@shoelace-style/shoelace/dist/components/divider/divider.js';
import '@shoelace-style/shoelace/dist/components/icon/icon.js';
import '@shoelace-style/shoelace/dist/components/menu-item/menu-item.js';
import SlMenuItem from '@shoelace-style/shoelace/dist/components/menu-item/menu-item.js';
import '@shoelace-style/shoelace/dist/components/menu/menu.js';
import '@shoelace-style/shoelace/dist/components/tab-group/tab-group.js';
import '@shoelace-style/shoelace/dist/components/tab-panel/tab-panel.js';
import '@shoelace-style/shoelace/dist/components/tab/tab.js';
import {
	Router,
	Routes,
	appClientContext,
	notify,
	notifyError,
	wrapPathInSvg,
} from '@tnesh-stack/elements';
import '@tnesh-stack/elements/dist/elements/display-error.js';
import { SignalWatcher } from '@tnesh-stack/signals';
import { LitElement, css, html } from 'lit';
import { customElement } from 'lit/decorators.js';

import { appStyles } from './app-styles.js';
import {
	adminWebsocketContext,
	isMobileContext,
	rootRouterContext,
} from './context.js';

@customElement('home-page')
export class HomePage extends SignalWatcher(LitElement) {
	@consume({ context: appClientContext, subscribe: true })
	client!: AppClient;

	@consume({ context: adminWebsocketContext, subscribe: true })
	adminWebsocket!: AdminWebsocket;

	@consume({ context: profilesProviderContext, subscribe: true })
	profilesProvider!: ProfilesProvider;

	@consume({ context: messengerStoreContext, subscribe: true })
	messengerStore!: MessengerStore;

	@consume({ context: friendsStoreContext, subscribe: true })
	friendsStore!: FriendsStore;

	@consume({ context: rootRouterContext, subscribe: true })
	rootRouter!: Router;

	routes = new Routes(this, [
		{
			path: '',
			render: () => this.renderPlaceholder(),
		},
		{
			path: 'peer-chat/:peerChatHash',
			render: ({ peerChatHash }) =>
				html`<peer-chat
					.peerChatHash=${decodeHashFromBase64(peerChatHash!)}
					style="flex: 1;"
				>
					${this.isMobile
						? html`
								<sl-icon-button
									slot="top-bar-left-action"
									.src=${wrapPathInSvg(mdiArrowLeft)}
									@click=${() => this.rootRouter.goto('')}
								></sl-icon-button>
							`
						: html``}
				</peer-chat>`,
		},
		{
			path: 'peer/:peer',
			render: ({ peer }) =>
				html`<peer-chat .peer=${decodeHashFromBase64(peer!)} style="flex: 1;">
					${this.isMobile
						? html`
								<sl-icon-button
									slot="top-bar-left-action"
									.src=${wrapPathInSvg(mdiArrowLeft)}
									@click=${() => this.rootRouter.goto('')}
								></sl-icon-button>
							`
						: html``}
				</peer-chat>`,
		},
		{
			path: 'group-chat/:groupChatHash',
			render: ({ groupChatHash }) =>
				html`<group-chat
					.groupChatHash=${decodeHashFromBase64(groupChatHash!)}
					style="flex: 1;"
				>
					${this.isMobile
						? html`
								<sl-icon-button
									slot="top-bar-left-action"
									.src=${wrapPathInSvg(mdiArrowLeft)}
									@click=${() => this.rootRouter.goto('')}
								></sl-icon-button>
							`
						: html``}
				</group-chat>`,
		},
	]);

	renderPlaceholder() {
		return html`<div
			class="column placeholder"
			style="flex: 1; align-items: center; justify-content: center; gap: 8px"
		>
			<sl-icon
				.src=${wrapPathInSvg(mdiChatOutline)}
				style="height: 64px; width: 64px"
			>
			</sl-icon>
			<span>${msg('Select a chat.')}</span>
		</div>`;
	}

	renderHomePanel() {
		const pendingFriendRequests = this.friendsStore.pendingFriendRequests.get();
		const pendingFriendRequestsCount =
			pendingFriendRequests.status !== 'completed'
				? 0
				: Object.values(pendingFriendRequests.value).filter(
						pendingFriendRequest =>
							pendingFriendRequest.event.content.to_agents.find(
								a =>
									encodeHashToBase64(a) ===
									encodeHashToBase64(this.friendsStore.client.client.myPubKey),
							),
					).length;
		const allChats = this.messengerStore.allChats.get();
		const newChatActivityCount =
			allChats.status !== 'completed'
				? 0
				: allChats.value.reduce(
						(acc, next) => acc + next.myUnreadMessages.length,
						0,
					);

		return html`
			<sl-tab-group placement="bottom" style="flex: 1; margin: 0 8px">
				<sl-tab style="flex: 1" slot="nav" panel="all_chats">
					<div class="row" style="justify-content: center; flex: 1">
						<div class="column" style="align-items: center; gap: 4px;">
							<sl-icon
								.src=${wrapPathInSvg(mdiChat)}
								style="font-size: 24px"
							></sl-icon>
							<span> ${msg('Chats')} </span>
						</div>
						${newChatActivityCount > 0
							? html`
									<sl-badge
										variant="primary"
										pill
										pulse
										style="align-self: center;"
										>${newChatActivityCount}</sl-badge
									>
								`
							: html``}
					</div>
				</sl-tab>
				<sl-tab style="flex: 1" slot="nav" panel="my_friends">
					<div class="row" style="justify-content: center; flex: 1">
						<div class="column" style="align-items: center; gap: 4px;">
							<sl-icon
								.src=${wrapPathInSvg(mdiAccountGroup)}
								style="font-size: 24px"
							></sl-icon>
							<span> ${msg('Friends')} </span>
						</div>
						${pendingFriendRequestsCount > 0
							? html`
									<sl-badge
										variant="primary"
										pill
										pulse
										style="align-self: center;"
										>${pendingFriendRequestsCount}</sl-badge
									>
								`
							: html``}
					</div></sl-tab
				>
				<sl-tab-panel name="all_chats">
					<all-chats
						style="min-height: 100%; margin: 8px"
						@group-chat-selected=${(e: CustomEvent) => {
							this.routes.goto(
								`group-chat/${encodeHashToBase64(e.detail.groupChatHash)}`,
							);
						}}
						@peer-chat-selected=${(e: CustomEvent) => {
							this.routes.goto(
								`peer-chat/${encodeHashToBase64(e.detail.peerChatHash)}`,
							);
						}}
					>
					</all-chats>
					<sl-button
						style="position: absolute; display: none; bottom: 16px; right: 16px"
						variant="primary"
						circle
						@click=${() =>
							this.dispatchEvent(
								new CustomEvent('new-message-clicked', {
									bubbles: true,
									composed: true,
								}),
							)}
						><sl-icon .src=${wrapPathInSvg(mdiMessagePlus)}></sl-icon
					></sl-button>
				</sl-tab-panel>
				<sl-tab-panel name="my_friends">
					<sl-dialog id="add-friend-dialog" .label=${msg('Add friend')}>
						<div class="column" style="gap: 16px">
							<span
								>${msg(
									'Ask your friend scan this QR code to send you a friend request.',
								)}
							</span>
							<div class="column" style="align-items: center;">
								<friend-request-qr-code style="align-self: center" size="256">
								</friend-request-qr-code>
							</div>
						</div>
						${this.isMobile
							? html`
									<sl-button
										variant="primary"
										slot="footer"
										@click=${async (e: CustomEvent) => {
											const button = e.target as SlButton;
											button.loading = true;
											try {
												await scanQrCodeAndSendFriendRequest(this.friendsStore);
												(
													this.shadowRoot!.getElementById(
														'add-friend-dialog',
													) as SlDialog
												).hide();
												notify(msg('Friend request sent.'));
											} catch (e) {
												console.error(e);
												notifyError(msg('Failed to send friend request.'));
											}
											button.loading = false;
										}}
										>${msg('Scan QR Code')}
									</sl-button>
								`
							: html``}
					</sl-dialog>
					<div class="column" style="gap: 16px; min-height: 100%; margin: 8px">
						${pendingFriendRequestsCount > 0
							? html`
									<sl-card>
										<div class="column" style="gap: 24px; flex: 1">
											<span class="title">${msg('Friend requests')}</span>
											<friend-requests> </friend-requests>
										</div>
									</sl-card>
								`
							: html``}

						<my-friends
							style="flex: 1"
							@friend-clicked=${(e: CustomEvent) =>
								this.routes.goto(
									`peer/${encodeHashToBase64(e.detail.agents[0])}`,
								)}
						>
						</my-friends>

						<sl-button
							pill
							variant="primary"
							style="position: absolute; right: 16px; bottom: 16px"
							@click=${() =>
								(
									this.shadowRoot!.getElementById(
										'add-friend-dialog',
									) as SlDialog
								).show()}
						>
							<sl-icon
								slot="prefix"
								.src=${wrapPathInSvg(mdiAccountPlus)}
							></sl-icon>
							${msg('Add Friend')}
						</sl-button>
					</div>
				</sl-tab-panel>
			</sl-tab-group>
		`;
	}

	// 		</sl-tab-panel>
	// 		<sl-tab-panel name="all_profiles">
	// 			<all-profiles
	// 				.excludedProfiles=${this.myProfile.status === 'completed' &&
	// 				this.myProfile.value
	// 					? [this.myProfile.value.profileHash]
	// 					: []}
	// 				style="flex: 1; margin: 8px"
	// 				@profile-selected=${async (e: CustomEvent) => {
	// 					const agents = await toPromise(
	// 						this.profilesStore.agentsForProfile.get(e.detail.profileHash),
	// 					);

	// 					if (agents.length > 0) {
	// 						this.routes.goto(`peer/${encodeHashToBase64(agents[0])}`);
	// 					} else {
	// 						const profile = await toPromise(
	// 							this.profilesStore.profiles.get(e.detail.profileHash)
	// 								.original,
	// 						);

	// 						if (profile) {
	// 							this.routes.goto(
	// 								`peer/${encodeHashToBase64(profile.action.author)}`,
	// 							);
	// 						} else {
	// 							const profile = await toPromise(
	// 								this.profilesStore.profiles.get(e.detail.profileHash)
	// 									.latestVersion,
	// 							);
	// 							this.routes.goto(
	// 								`peer/${encodeHashToBase64(profile.action.author)}`,
	// 							);
	// 						}
	// 					}
	// 				}}
	// 			>
	// 			</all-profiles>
	// 		</sl-tab-panel>
	// 	</sl-tab-group>

	@consume({ context: isMobileContext })
	isMobile!: boolean;

	renderActions() {
		if (this.isMobile) {
			return html`
				<sl-dropdown>
					<sl-icon-button
						slot="trigger"
						style="font-size: 24px;"
						.src=${wrapPathInSvg(mdiDotsVertical)}
					></sl-icon-button>
					<sl-menu
						@sl-select=${async (e: CustomEvent) => {
							const item = e.detail.item as SlMenuItem;
							const value = item.value;
							if (value === 'new_group') {
								this.dispatchEvent(
									new CustomEvent('create-group-chat-clicked'),
								);
							} else if (value === 'my_profile') {
								this.dispatchEvent(new CustomEvent('profile-clicked'));
							}
						}}
					>
						<sl-menu-item value="new_group">
							<sl-icon
								.src=${wrapPathInSvg(mdiAccountMultiplePlus)}
								slot="prefix"
							></sl-icon>
							${msg('New Group')}</sl-menu-item
						>
						<sl-menu-item value="my_profile">
							<sl-icon
								.src=${wrapPathInSvg(mdiAccount)}
								slot="prefix"
							></sl-icon>
							${msg('My Profile')}</sl-menu-item
						>
					</sl-menu>
				</sl-dropdown>
			`;
		}

		return html`
			<div class="row" style="gap: 16px; align-items: center">
				<sl-button
					style="font-size: 24px"
					@click=${() =>
						this.dispatchEvent(new CustomEvent('create-group-chat-clicked'))}
					outline
					><sl-icon
						slot="prefix"
						.src=${wrapPathInSvg(mdiAccountMultiplePlus)}
					></sl-icon
					>${msg('New Group')}
				</sl-button>

				<agent-avatar
					@click=${() =>
						this.dispatchEvent(
							new CustomEvent('profile-clicked', {
								detail: true,
								composed: true,
							}),
						)}
					.agentPubKey=${this.client.myPubKey}
				></agent-avatar>
			</div>
		`;
	}

	renderMobile() {
		if (this.routes.currentPathname() !== '') return this.routes.outlet();
		return html`
			<div class="column" style="flex: 1">
				<div class="row top-bar">
					<span class="title" style="flex: 1">${msg('Dash Chat')}</span>

					${this.renderActions()}
				</div>
				${this.renderHomePanel()}
			</div>
		`;
	}

	renderDesktop() {
		return html`
			<div class="column" style="flex: 1">
				<div class="row top-bar">
					<span class="title" style="flex: 1">${msg('Dash Chat')}</span>

					${this.renderActions()}
				</div>
				<div class="row" style="flex: 1">
					<div class="column" style="flex-basis: 400px">
						${this.renderHomePanel()}
					</div>

					<sl-divider vertical style="--spacing: 0"> </sl-divider>

					${this.routes.outlet()}
				</div>
			</div>
		`;
	}

	render() {
		if (this.isMobile) return this.renderMobile();
		return this.renderDesktop();
	}

	static styles = [
		css`
			:host {
				display: flex;
				flex: 1;
			}
			sl-tab-group {
				display: flex;
			}
			sl-tab-group::part(base) {
				flex: 1;
			}
			sl-tab-group::part(body) {
				flex: 1;
			}
			sl-tab {
				display: flex;
			}
			sl-tab::part(base) {
				flex: 1;
			}
			sl-divider {
				margin-right: 0;
			}
			sl-tab-panel {
				position: relative;
				height: 100%;
			}
			sl-tab-panel::part(base) {
				height: 100%;
			}
			group-chat::part(chat) {
				margin: 8px;
				margin-top: 0px;
			}
			peer-chat::part(chat) {
				margin: 8px;
				margin-top: 0px;
			}
		`,
		...appStyles,
	];
}
