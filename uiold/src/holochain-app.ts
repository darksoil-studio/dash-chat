import { DeepLinkApi } from '@darksoil-studio/friends-zome';
import '@darksoil-studio/friends-zome/dist/elements/deep-link-api-context.js';
import '@darksoil-studio/friends-zome/dist/elements/friends-context.js';
import '@darksoil-studio/friends-zome/dist/elements/profile-prompt.js';
import '@darksoil-studio/friends-zome/dist/elements/select-friend.js';
import '@darksoil-studio/friends-zome/dist/elements/update-profile.js';
import '@darksoil-studio/holochain-elements/dist/elements/app-client-context.js';
import '@darksoil-studio/holochain-elements/dist/elements/display-error.js';
import { SignalWatcher } from '@darksoil-studio/holochain-signals';
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
import { mdiLink } from '@mdi/js';
import '@saurl/tauri-plugin-safe-area-insets-css-api';
import {
	getBottomInset,
	getTopInset,
	onKeyboardHidden,
	onKeyboardShown,
} from '@saurl/tauri-plugin-safe-area-insets-css-api';
import '@shoelace-style/shoelace/dist/components/button/button.js';
import '@shoelace-style/shoelace/dist/components/dialog/dialog.js';
import '@shoelace-style/shoelace/dist/components/icon-button/icon-button.js';
import '@shoelace-style/shoelace/dist/components/icon/icon.js';
import '@shoelace-style/shoelace/dist/components/spinner/spinner.js';
import { onOpenUrl } from '@tauri-apps/plugin-deep-link';
import Emittery from 'emittery';
import { LitElement, css, html } from 'lit';
import { customElement, property, state } from 'lit/decorators.js';

import { appStyles } from './app-styles.js';
import './automatic-update-dialog.js';
import { adminWebsocketContext, isMobileContext } from './context.js';
import './home-page.js';
import './link-device-dialog.js';
import './overlay-page.js';
import './splash-screen.js';
import { splascreenCompleted } from './splash-screen.js';
import {
	connectConsoleToTauriLogs,
	getOS,
	sleep,
	withRetries,
} from './utils.js';

export const MOBILE_WIDTH_PX = 600;

@localized()
@customElement('holochain-app')
export class HolochainApp extends SignalWatcher(LitElement) {
	@state()
	_loading = true;
	@state()
	_splashscreen = false;
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

	// emitter = new Emittery();
	// deepLinkApi: DeepLinkApi = {
	// 	buildDeepLink: code => `dash-chat://${code}`,
	// 	onDeepLinkClicked: handler => {
	// 		this.emitter.on('deep-link', (deepLinkUrl: string) => {
	// 			if (!deepLinkUrl.startsWith('dash-chat://')) {
	// 				console.warn('Received an unknown deep link: ', deepLinkUrl);
	// 				return;
	// 			}
	// 			const split = deepLinkUrl.split('dash-chat://');
	// 			handler(split[1]);
	// 		});
	// 	},
	// };

	async firstUpdated() {
		connectConsoleToTauriLogs();
		onKeyboardShown(async () => {
			const bi = await getBottomInset();
			const top = await getTopInset();
			document.documentElement.style = `--safe-area-inset-bottom: ${bi?.inset}px; --safe-area-inset-top: ${top?.inset}px`;
		});

		await onOpenUrl(urls => {
			console.log('Received deep links:', urls);
			for (const url of urls) {
				this.emitter.emit('deep-link', url);
			}
		});

		this._isMobile = this.getBoundingClientRect().width < MOBILE_WIDTH_PX;
		new ResizeController(this, {
			callback: () => {
				this._isMobile = this.getBoundingClientRect().width < MOBILE_WIDTH_PX;
			},
		});
		this._splashscreen = splascreenCompleted();
		this._loading = true;
		this._error = undefined;

		try {
			this._client = await withRetries(() =>
				AppWebsocket.connect({
					defaultTimeout: 100_000,
				}),
			);
		} catch (e: unknown) {
			this._error = e;
		} finally {
			this._loading = false;
		}
		await sleep(100);
		await this._client
			.callZome({
				role_name: 'main',
				zome_name: 'safehold_async_messages',
				fn_name: 'receive_messages',
			})
			.catch(console.warn);
		await sleep(1000);
		await this._client
			.callZome({
				role_name: 'main',
				zome_name: 'safehold_async_messages',
				fn_name: 'receive_messages',
			})
			.catch(console.warn);

		this.receiveMessages();
	}

	async receiveMessages() {
		try {
			await this._client.callZome({
				role_name: 'main',
				zome_name: 'safehold_async_messages',
				fn_name: 'receive_messages',
			});
		} catch (e) {
			console.error('Failed to receive_messages', e);
		} finally {
			setTimeout(() => {
				this.receiveMessages();
			}, 10_000);
		}
	}

	render() {
		if (this._splashscreen) {
			return html`<splash-screen
				style="flex: 1; margin-top: var(--safe-area-inset-top)"
				@start-app-clicked=${() => this.firstUpdated()}
			></splash-screen>`;
		}
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
						.headline=${msg('Error connecting to holochain.')}
					>
					</display-error>
				</div>
			`;
		}

		return html`
			<automatic-update-dialog> </automatic-update-dialog>
			<app-client-context .client=${this._client}>
				<notifications-context role="main">
					<messenger-context role="main">
						<linked-devices-context role="main">
								<friends-context role="main">
									<profile-prompt style="flex: 1;">
										<home-page> </home-page>
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
				position: fixed;
				top: 0;
				left: 0;
				right: 0;
				height: calc(100vh - var(--safe-area-inset-bottom, 0px));
			}
		`,
		...appStyles,
	];
}
