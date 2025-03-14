import { scanQrCodeAndSendFriendRequest } from '@darksoil-studio/friends-zome/dist/elements/friend-request-qr-code.js';
import '@darksoil-studio/friends-zome/dist/elements/friend-request-qr-code.js';
import '@darksoil-studio/friends-zome/dist/elements/friend-requests.js';
import '@darksoil-studio/friends-zome/dist/elements/friends-context.js';
import { FriendsContext } from '@darksoil-studio/friends-zome/dist/elements/friends-context.js';
import '@darksoil-studio/friends-zome/dist/elements/my-friends.js';
import '@darksoil-studio/friends-zome/dist/elements/profile-prompt.js';
import '@darksoil-studio/friends-zome/dist/elements/select-friend.js';
import '@darksoil-studio/linked-devices-zome/dist/elements/linked-devices-context.js';
import '@darksoil-studio/messenger-zome/dist/elements/create-group-chat.js';
import '@darksoil-studio/messenger-zome/dist/elements/messenger-context.js';
import '@darksoil-studio/notifications-zome/dist/elements/notifications-context.js';
import '@darksoil-studio/profiles-provider/dist/elements/my-profile.js';
import {
	AdminWebsocket,
	AppClient,
	AppWebsocket,
	encodeHashToBase64,
} from '@holochain/client';
import { ResizeController } from '@lit-labs/observers/resize-controller.js';
import { provide } from '@lit/context';
import { localized, msg } from '@lit/localize';
import { mdiAccountPlus, mdiArrowLeft, mdiLink } from '@mdi/js';
import '@shoelace-style/shoelace/dist/components/button/button.js';
import '@shoelace-style/shoelace/dist/components/dialog/dialog.js';
import SlDialog from '@shoelace-style/shoelace/dist/components/dialog/dialog.js';
import '@shoelace-style/shoelace/dist/components/icon-button/icon-button.js';
import '@shoelace-style/shoelace/dist/components/icon/icon.js';
import '@shoelace-style/shoelace/dist/components/spinner/spinner.js';
import { Router, hashState, wrapPathInSvg } from '@tnesh-stack/elements';
import '@tnesh-stack/elements/dist/elements/app-client-context.js';
import '@tnesh-stack/elements/dist/elements/display-error.js';
import { SignalWatcher } from '@tnesh-stack/signals';
import { LitElement, css, html } from 'lit';
import { customElement, property, state } from 'lit/decorators.js';
import { styleMap } from 'lit/directives/style-map.js';

import { appStyles } from './app-styles.js';
import './automatic-update-dialog.js';
import {
	adminWebsocketContext,
	isMobileContext,
	rootRouterContext,
} from './context.js';
import './home-page.js';
import { LinkDeviceDialog } from './link-device-dialog.js';
import './link-device-dialog.js';
import './overlay-page.js';

export const MOBILE_WIDTH_PX = 600;

@localized()
@customElement('holochain-app')
export class HolochainApp extends SignalWatcher(LitElement) {
	@state()
	_loading = true;
	@state()
	_view = { view: 'main' };
	@state()
	_error: unknown | undefined;

	_client!: AppClient;

	@provide({ context: adminWebsocketContext })
	@property()
	_adminWs!: AdminWebsocket;

	@provide({ context: isMobileContext })
	@property()
	_isMobile: boolean = false;

	@provide({ context: rootRouterContext })
	router = new Router(this, [
		{
			path: '',
			enter: () => {
				// Redirect to "/home/"
				this.router.goto('/home/');
				return false;
			},
		},
		{
			path: '/',
			enter: () => {
				// Redirect to "/home/"
				this.router.goto('/home/');
				return false;
			},
		},
		{
			path: '/home/*',
			render: () =>
				html`<home-page
					@profile-clicked=${() => this.router.goto('/my-profile')}
					@create-group-chat-clicked=${() =>
						this.router.goto('/create-group-chat')}
					@my-friends-clicked=${() => this.router.goto('/my-friends')}
					@new-message-clicked=${() => this.router.goto('/new-message')}
				></home-page>`,
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
			path: '/my-profile',
			render: () => this.renderMyProfilePage(),
		},
		{
			path: '/new-message',
			render: () => html`
				<overlay-page
					.title=${msg('New message')}
					icon="back"
					@close-requested=${() => this.router.goto('/home/')}
				>
					<select-friend
						style="flex: 1;"
						@friend-selected=${(e: CustomEvent) => {
							this.router.goto(
								`/home/peer/${encodeHashToBase64(e.detail.friend.agents[0])}`,
							);
						}}
					>
					</select-friend>
				</overlay-page>
			`,
		},
		{
			path: '/create-group-chat',
			render: () => html`
				<overlay-page
					.title=${msg('New group chat')}
					icon="back"
					@close-requested=${() => this.router.goto('/home/')}
				>
					<sl-card>
						<div class="column" style="gap: 12px; flex: 1">
							<span class="title">${msg('New group chat')} </span>
							<create-group-chat
								style="flex: 1;"
								@group-chat-created=${(e: CustomEvent) => {
									this.router.goto(
										`/home/group-chat/${encodeHashToBase64(e.detail.groupChatHash)}`,
									);
								}}
							>
							</create-group-chat>
						</div>
					</sl-card>
				</overlay-page>
			`,
		},
	]);

	async firstUpdated() {
		new ResizeController(this, {
			callback: () => {
				this._isMobile = this.getBoundingClientRect().width < MOBILE_WIDTH_PX;
			},
		});

		try {
			this._client = await AppWebsocket.connect({
				defaultTimeout: 300_000,
			});
			// this._adminWs = await AdminWebsocket.connect();
		} catch (e: unknown) {
			this._error = e;
		} finally {
			this._loading = false;
		}
	}

	renderMyProfilePage() {
		return html`
			<link-device-dialog id="link-device-dialog"> </link-device-dialog>
			<overlay-page
				.title=${msg('My profile')}
				icon="back"
				@close-requested=${() => this.router.goto('/home/')}
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
					<div class="column" style=" gap: 32px; flex: 1">
						<span class="title">${msg('My profile')}</span>
						<my-profile style="margin: 8px; flex: 1"></my-profile>
					</div>
				</sl-card>
			</overlay-page>
		`;
	}

	render() {
		if (this._loading) {
			return html`<div
				class="row"
				style="flex: 1; height: 100%; align-items: center; justify-content: center;"
			>
				<sl-spinner style="font-size: 2rem"></sl-spinner>
			</div>`;
		}

		if (this._error) {
			return html`
				<div
					style="flex: 1; height: 100%; align-items: center; justify-content: center;"
				>
					<display-error
						.error=${this._error}
						.headline=${msg('Error connecting to holochain')}
					>
					</display-error>
				</div>
			`;
		}

		return html`
			<automatic-update-dialog> </automatic-update-dialog>
			<app-client-context .client=${this._client}>
				<notifications-context role="messenger_demo">
					<messenger-context role="messenger_demo">
						<linked-devices-context role="messenger_demo">
							<friends-context role="messenger_demo">
								<profile-prompt style="flex: 1;">
									${this.router.outlet()}
								</profile-prompt>
							</friends-context>
						</linked-devices-context>
					</messenger-context>
				</notifications-context>
			</app-client-context>
		`;
	}

	static styles = [
		css`
			:host {
				display: flex;
				flex: 1;
			}
		`,
		...appStyles,
	];
}
