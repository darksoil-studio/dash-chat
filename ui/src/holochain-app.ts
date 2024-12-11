import '@darksoil-studio/file-storage-zome/dist/elements/file-storage-context.js';
import '@darksoil-studio/linked-devices-zome/dist/elements/linked-devices-context.js';
import '@darksoil-studio/messenger-zome/dist/elements/create-group-chat.js';
import '@darksoil-studio/messenger-zome/dist/elements/messenger-context.js';
import '@darksoil-studio/profiles-zome/dist/elements/my-profile.js';
import '@darksoil-studio/profiles-zome/dist/elements/profile-prompt.js';
import '@darksoil-studio/profiles-zome/dist/elements/profiles-context.js';
import {
	AdminWebsocket,
	AppClient,
	AppWebsocket,
	encodeHashToBase64,
} from '@holochain/client';
import { ResizeController } from '@lit-labs/observers/resize-controller.js';
import { provide } from '@lit/context';
import { localized, msg } from '@lit/localize';
import { mdiArrowLeft, mdiLink } from '@mdi/js';
import '@shoelace-style/shoelace/dist/components/button/button.js';
import '@shoelace-style/shoelace/dist/components/dialog/dialog.js';
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
import {
	adminWebsocketContext,
	isMobileContext,
	rootRouterContext,
} from './context.js';
import './home-page.js';
import { LinkDeviceDialog } from './link-device-dialog.js';
import './link-device-dialog.js';

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
					@create-group-chat-selected=${() =>
						this.router.goto('/create-group-chat')}
				></home-page>`,
		},
		{
			path: '/my-profile',
			render: () => this.renderMyProfilePage(),
		},
		{
			path: '/create-group-chat',
			render: () => html`
				<div class="column fill">
					<div class="row top-bar">
						<sl-icon-button
							style="color: black"
							.src=${wrapPathInSvg(mdiArrowLeft)}
							@click=${() => this.router.goto('/home/')}
						></sl-icon-button>
						<span class="title" style="flex: 1"
							>${msg('Create Group Chat')}</span
						>
					</div>
					<div class="row" style="justify-content: center; flex: 1">
						<sl-card style="margin: 16px; flex-basis: 500px">
							<div class="column" style="gap: 12px; flex: 1">
								<span class="title">${msg('New Group Chat')} </span>
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
					</div>
				</div>
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
			this._client = await AppWebsocket.connect();
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
			<div class="column fill">
				<div class="row top-bar" style="gap: 8px">
					<sl-icon-button
						style="color: black"
						.src=${wrapPathInSvg(mdiArrowLeft)}
						@click=${() => this.router.goto('/home/')}
					></sl-icon-button>
					<span class="title" style="flex: 1">${msg('My Profile')}</span>

					<div style="flex: 1"></div>

					<sl-button
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
				</div>

				<sl-card
					style=${styleMap({
						width: this._isMobile ? 'unset' : '600px',
						margin: '24px',
						'align-self': this._isMobile ? 'auto' : 'center',
					})}
				>
					<my-profile style="margin: 16px; flex: 1"></my-profile>
				</sl-card>
			</div>
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
			<app-client-context .client=${this._client}>
				<messenger-context role="messenger_demo">
					<file-storage-context role="messenger_demo">
						<linked-devices-context role="messenger_demo">
							<profiles-context role="messenger_demo">
								<profile-prompt style="flex: 1;">
									${this.router.outlet()}
								</profile-prompt>
							</profiles-context>
						</linked-devices-context>
					</file-storage-context>
				</messenger-context>
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
