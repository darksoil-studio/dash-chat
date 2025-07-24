import {
	FriendsStore,
	friendsStoreContext,
} from '@darksoil-studio/friends-zome';
import '@darksoil-studio/friends-zome/dist/elements/friend-request-qr-code.js';
import { scanQrCodeAndSendFriendRequest } from '@darksoil-studio/friends-zome/dist/elements/friend-request-qr-code.js';
import '@darksoil-studio/friends-zome/dist/elements/friend-requests.js';
import '@darksoil-studio/friends-zome/dist/elements/my-friends.js';
import {
	Router,
	Routes,
	appClientContext,
	notify,
	notifyError,
	wrapPathInSvg,
} from '@darksoil-studio/holochain-elements';
import '@darksoil-studio/holochain-elements/dist/elements/display-error.js';
import { SignalWatcher } from '@darksoil-studio/holochain-signals';
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
	mdiLink,
	mdiMessagePlus,
} from '@mdi/js';
import { SlButton, SlDialog } from '@shoelace-style/shoelace';
import '@shoelace-style/shoelace/dist/components/badge/badge.js';
import '@shoelace-style/shoelace/dist/components/button/button.js';
import '@shoelace-style/shoelace/dist/components/divider/divider.js';
import '@shoelace-style/shoelace/dist/components/icon-button/icon-button.js';
import '@shoelace-style/shoelace/dist/components/icon/icon.js';
import '@shoelace-style/shoelace/dist/components/menu-item/menu-item.js';
import SlMenuItem from '@shoelace-style/shoelace/dist/components/menu-item/menu-item.js';
import '@shoelace-style/shoelace/dist/components/menu/menu.js';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import {
	checkPermissions,
	openAppSettings,
	requestPermissions,
} from '@tauri-apps/plugin-barcode-scanner';
import { Options, onAction } from '@tauri-apps/plugin-notification';
import { LitElement, css, html } from 'lit';
import { customElement } from 'lit/decorators.js';

import { appStyles } from './app-styles.js';
import { adminWebsocketContext, isMobileContext } from './context.js';
import { LinkDeviceDialog } from './link-device-dialog.js';
import { isMobileOs } from './utils.js';

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

	router = new Router(this, [
		{
			path: '/',
			render: () => this.renderPlaceholder(),
		},
		{
			path: '/my-profile',
			render: () => html`
				<link-device-dialog id="link-device-dialog"> </link-device-dialog>
				<overlay-page
					.title=${msg('My profile')}
					icon="back"
					@close-requested=${() => this.router.goto('/')}
				>
					<sl-button
						outline
						slot="action"
						@click=${() => {
							const dialog = this.shadowRoot!.getElementById(
								'link-device-dialog',
							) as LinkDeviceDialog;
							dialog.show();
						}}
					>
						<sl-icon .src=${wrapPathInSvg(mdiLink)}></sl-icon>
						${msg('Link Device')}
					</sl-button>

					<sl-card>
						<div class="column" style="gap: 16px; flex: 1">
							<span class="title">${msg('My profile')}</span>
							<my-profile
								style="margin: 8px; flex: 1"
								@edit-profile-clicked=${() =>
									this.router.goto('/my-profile/edit')}
							></my-profile>
						</div>
					</sl-card>
				</overlay-page>
			`,
		},
		{
			path: '/my-profile/edit',
			render: () => html`
				<overlay-page
					.title=${msg('Edit profile')}
					@close-requested=${() => this.router.goto('/my-profile')}
				>
					<sl-card>
						<div class="column" style="gap: 16px; flex: 1">
							<span class="title">${msg('Edit profile')}</span>
							<update-profile
								style="margin: 8px; flex: 1"
								@profile-updated=${() => this.router.goto('/my-profile')}
							></update-profile>
						</div>
					</sl-card>
				</overlay-page>
			`,
		},
		{
			path: '/friend-requests',
			enter: () => {
				// Redirect to "/home/"
				this.router.goto('/my-friends');
				return false;
			},
		},
		{
			path: '/my-friends',
			render: () => this.renderFriends(),
		},
		{
			path: '/new-message',
			render: () => html`
				<overlay-page
					.title=${msg('New message')}
					icon="back"
					@close-requested=${() => this.router.goto('/')}
				>
					<div class="column" style="gap: 16px">
						${this.renderAddFriendDialog()}
						<sl-button
							size="large"
							class="no-border"
							@click=${() =>
								(
									this.shadowRoot!.getElementById(
										'add-friend-dialog',
									) as SlDialog
								).show()}
						>
							<sl-icon
								style="font-size: 1.4rem"
								slot="prefix"
								.src=${wrapPathInSvg(mdiAccountPlus)}
							>
							</sl-icon
							>${msg('Add friend')}
						</sl-button>

						<sl-button
							class="no-border"
							size="large"
							@click=${() => this.router.goto('/create-group-chat')}
						>
							<sl-icon
								slot="prefix"
								.src=${wrapPathInSvg(mdiAccountGroup)}
								style="font-size: 1.4rem"
							>
							</sl-icon
							>${msg('New group')}
						</sl-button>

						<sl-divider> </sl-divider>

						<span style="font-size: 20px">${msg('Friends')}</span>

						<select-friend
							style="flex: 1;"
							@friend-selected=${(e: CustomEvent) => {
								this.router.goto(
									`/peer/${encodeHashToBase64(e.detail.agents[0])}`,
								);
							}}
						>
						</select-friend>
					</div>
				</overlay-page>
			`,
		},
		{
			path: '/create-group-chat',
			render: () => html`
				<overlay-page
					.title=${msg('New group chat')}
					icon="back"
					@close-requested=${() => this.router.goto('/new-message')}
				>
					<sl-card>
						<div class="column" style="gap: 16px; flex: 1">
							<span class="title">${msg('New group chat')}</span>
							<create-group-chat
								style="flex: 1;"
								@group-chat-created=${(e: CustomEvent) => {
									this.router.goto(
										`/group-chat/${encodeHashToBase64(e.detail.groupChatHash)}`,
									);
								}}
							>
							</create-group-chat>
						</div>
					</sl-card>
				</overlay-page>
			`,
		},
		{
			path: '/peer-chat/:peerChatHash',
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
									@click=${() => this.router.goto('/')}
								></sl-icon-button>
							`
						: html``}
				</peer-chat>`,
		},
		{
			path: '/peer/:peer',
			render: ({ peer }) =>
				html`<peer-chat .peer=${decodeHashFromBase64(peer!)} style="flex: 1;">
					${this.isMobile
						? html`
								<sl-icon-button
									slot="top-bar-left-action"
									.src=${wrapPathInSvg(mdiArrowLeft)}
									@click=${() => this.router.goto('/')}
								></sl-icon-button>
							`
						: html``}
				</peer-chat>`,
		},
		{
			path: '/group-chat/:groupChatHash',
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
									@click=${() => this.router.goto('/')}
								></sl-icon-button>
							`
						: html``}
				</group-chat>`,
		},
	]);

	async firstUpdated() {
		console.warn('heyi');
		if (!isMobileOs()) return;
		console.warn('hey2');

		listen('notification://action-performed', e => {
			const notification = (e.payload as any).notification as Options;
			this.handleNotificationClicked(notification);
		});

		console.warn(`before`)
		// If the app was launched from a notification, redirect to the appropriate URL
		const n: { notification: Options } | undefined = await invoke(
			'get_launching_notification_action',
		);
		console.warn(`aaa${n}`)

		if (n) {
			this.handleNotificationClicked(n.notification);
		}
	}

	handleNotificationClicked(notification: Options) {
		const group = notification.group;

		if (!group) {
			console.warn('Received a notification with no group.');
			return;
		}

		const split = group.split('/');
		if (split.length !== 2) {
			console.warn('Received a notification with a malformed group.');
			return;
		}

		const notificationType = split[0];

		switch (notificationType) {
			case 'friend-request':
				this.router.goto(`/my-friends`);
				return;
			case 'group-chat':
				this.router.goto(`/group-chat/${split[1]}`);
				return;
			case 'peer-chat':
				this.router.goto(`/peer-chat/${split[1]}`);
				return;
			default:
				console.warn(
					'Received a notification with an invalid group notification type: ',
					notificationType,
				);
		}
	}

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

	renderFriendsIcon() {
		const incomingFriendRequestsResult =
			this.friendsStore.incomingFriendRequests.get();
		const incomingFriendRequests =
			incomingFriendRequestsResult.status !== 'completed'
				? {}
				: incomingFriendRequestsResult.value;
		return html`
			<div style="position: relative">
				<sl-icon-button
					.src=${wrapPathInSvg(mdiAccountGroup)}
					@click=${() => this.router.goto('/my-friends')}
					style="font-size: 1.5rem"
				>
				</sl-icon-button>
				${Object.keys(incomingFriendRequests).length > 0
					? html`
							<sl-badge
								variant="primary"
								pill
								pulse
								style="position: absolute; right: -8px; bottom: -4px"
								>${Object.keys(incomingFriendRequests).length}</sl-badge
							>
						`
					: html``}
			</div>
		`;
	}

	renderAddFriendDialog() {
		return html`
			<sl-dialog id="add-friend-dialog" .label=${msg('Add friend')}>
				<div class="column" style="gap: 16px">
					<span
						>${msg(
							'Ask your friend scan this QR code to send you a friend request.',
						)}
					</span>
					<div class="column" style="align-items: center; padding: 16px">
						<friend-request-qr-code
							@friend-request-sent=${() => {
								(
									this.shadowRoot!.getElementById(
										'add-friend-dialog',
									) as SlDialog
								).hide();
								this.router.goto('/my-friends');
							}}
							style="align-self: center; "
							size="256"
							show-send-code-fallback
						>
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
									await this.scan();
									button.loading = false;
								}}
								>${msg('Scan QR Code')}
							</sl-button>
						`
					: html``}
			</sl-dialog>
		`;
	}

	renderFriends() {
		const pendingFriendRequestsResult =
			this.friendsStore.pendingFriendRequests.get();
		const pendingFriendRequests =
			pendingFriendRequestsResult.status !== 'completed'
				? {}
				: pendingFriendRequestsResult.value;
		const showFriendRequests = Object.keys(pendingFriendRequests).length > 0;
		return html`
			<overlay-page
				.title=${msg('My friends')}
				icon="back"
				@close-requested=${() => this.router.goto('/')}
			>
				<div class="column" style="gap: 24px; margin: 8px; flex: 1">
					${showFriendRequests
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
							this.router.goto(
								`/peer/${encodeHashToBase64(e.detail.agents[0])}`,
							)}
					>
					</my-friends>

					<sl-button
						pill
						variant="primary"
						size="large"
						style="position: absolute; right: 16px; bottom: 16px;"
						@click=${() =>
							(
								this.shadowRoot!.getElementById('add-friend-dialog') as SlDialog
							).show()}
					>
						<sl-icon
							slot="prefix"
							.src=${wrapPathInSvg(mdiAccountPlus)}
							style="font-size: 1.4rem"
						></sl-icon>
						${msg('Add friend')}
					</sl-button>
				</div>
				${this.renderAddFriendDialog()}
			</overlay-page>
		`;
	}

	renderChats() {
		return html`
			<div class="flex-scrollable-parent" style="flex: 1;  position: relative">
				<div class="flex-scrollable-container" style="flex:1">
					<div class="flex-scrollable-y" style="height: 100%;">
						<all-chats
							style="min-height: calc(100% - 48px); margin: 24px"
							@group-chat-selected=${(e: CustomEvent) => {
								this.router.goto(
									`/group-chat/${encodeHashToBase64(e.detail.groupChatHash)}`,
								);
							}}
							@peer-chat-selected=${(e: CustomEvent) => {
								this.router.goto(
									`/peer-chat/${encodeHashToBase64(e.detail.peerChatHash)}`,
								);
							}}
						>
						</all-chats>
						<sl-button
							style="position: absolute; bottom: 16px; right: 16px;"
							variant="primary"
							size="large"
							.pill=${!this.isMobile}
							.circle=${this.isMobile}
							@click=${() => this.router.goto('/new-message')}
						>
							${this.isMobile
								? html`
										<sl-icon
											.src=${wrapPathInSvg(mdiMessagePlus)}
											style="font-size: 1.5rem; vertical-align: -8px"
										></sl-icon>
									`
								: html`
										<sl-icon
											slot="prefix"
											.src=${wrapPathInSvg(mdiMessagePlus)}
											style="font-size: 1.3rem"
										></sl-icon>
										${msg('New message')}
									`}</sl-button
						>
					</div>
				</div>
			</div>
		`;
	}

	async scan() {
		try {
			const permission = await checkPermissions();
			if (permission === 'prompt') {
				await requestPermissions();
			} else if (permission === 'denied') {
				await openAppSettings();
			}
			await scanQrCodeAndSendFriendRequest(this.friendsStore);
			(this.shadowRoot!.getElementById('add-friend-dialog') as SlDialog).hide();
			notify(msg('Friend request sent.'));
			this.router.goto('/my-friends');
		} catch (e) {
			if ((e as any).message && (e as any).message.includes('permission')) {
				await openAppSettings();
				await this.scan();
			} else {
				console.error(e);
				notifyError(msg('Failed to send friend request.'));
			}
		}
	}

	@consume({ context: isMobileContext })
	isMobile!: boolean;

	renderActions() {
		if (this.isMobile) {
			return html`
				<div class="row" style="gap: 8px; align-items: center">
					${this.renderFriendsIcon()}
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
								if (value === 'my_profile') {
									this.router.goto('/my-profile');
								}
							}}
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
				</div>
			`;
		}

		return html`
			<div class="row" style="gap: 16px; align-items: center">
				${this.renderFriendsIcon()}
				<agent-avatar
					@click=${() => this.router.goto('/my-profile')}
					.agentPubKey=${this.client.myPubKey}
				></agent-avatar>
			</div>
		`;
	}

	renderMobile() {
		if (this.router.currentPathname() !== '/') return this.router.outlet();
		return html`
			<div class="column" style="flex: 1">
				<div class="row top-bar">
					<span class="title" style="flex: 1">${msg('Dash Chat')}</span>

					${this.renderActions()}
				</div>
				${this.renderChats()}
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
					<div class="column" style="flex-basis: 400px;">
						${this.renderChats()}
					</div>

					<sl-divider vertical style="--spacing: 0"> </sl-divider>

					${this.router.outlet()}
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
			sl-divider {
				margin-right: 0;
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
