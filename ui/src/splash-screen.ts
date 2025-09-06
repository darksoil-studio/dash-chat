import { notify, notifyError } from '@darksoil-studio/holochain-elements';
import { SignalWatcher } from '@darksoil-studio/holochain-signals';
import { AppWebsocket } from '@holochain/client';
import { consume } from '@lit/context';
import { localized, msg } from '@lit/localize';
import '@shoelace-style/shoelace/dist/components/button/button.js';
import SlButton from '@shoelace-style/shoelace/dist/components/button/button.js';
import '@shoelace-style/shoelace/dist/components/carousel-item/carousel-item.js';
import '@shoelace-style/shoelace/dist/components/carousel/carousel.js';
import '@tauri-apps/api';
import {
	isPermissionGranted,
	requestPermission,
} from '@tauri-apps/plugin-notification';
import { LitElement, css, html } from 'lit';
import { customElement, state } from 'lit/decorators.js';

import { appStyles } from './app-styles';
import { isMobileContext } from './context';
import { isMobileOs, sleep, withRetries, withTimeout } from './utils';

const SPLASHSCREEN_KEY = 'splashcreencompleted';

export function splascreenCompleted() {
	return !localStorage.getItem(SPLASHSCREEN_KEY);
}

export function completeSplascreen() {
	localStorage.setItem(SPLASHSCREEN_KEY, 'true');
}

@localized()
@customElement('splash-screen')
export class SplashScreen extends SignalWatcher(LitElement) {
	@state()
	initialized = false;

	@state()
	currentPage = 0;

	@consume({ context: isMobileContext })
	isMobile!: boolean;

	firstUpdated() {
		this.attemptConnect();
	}

	async attemptConnect() {
		try {
			const client = await AppWebsocket.connect();

			await sleep(200);

			await withRetries(
				async () =>
					withTimeout(
						() =>
							client.callZome({
								role_name: 'main',
								zome_name: 'messenger',
								fn_name: 'entry_defs',
								payload: undefined,
							}),
						1000,
					),
				10,
			);

			this.initialized = true;
		} catch (e: unknown) {
			console.error('Error calling init', e);
			setTimeout(() => this.attemptConnect(), 300);
		}
	}

	renderBackButton() {
		return html`
			<sl-button size="large" style="flex: 1" @click=${() => this.currentPage--}
				>${msg('Back')}</sl-button
			>
		`;
	}

	renderNextButton() {
		return html`
			<sl-button
				size="large"
				style="flex: 1"
				variant="primary"
				@click=${() => this.currentPage++}
				>${msg('Next')}</sl-button
			>
		`;
	}

	renderButtons() {
		return html`
			<div class="row" style="gap: 8px">
				${this.renderBackButton()} ${this.renderNextButton()}
			</div>
		`;
	}

	renderPage() {
		if (this.currentPage === 0) {
			return html`
				<div class="column" style="gap: 32px; flex: 1">
					<span class="title">${msg('Welcome to dash chat!')} </span>
					<span>${msg('A private peer-to-peer chat app.')} </span>

					<span style="flex: 1"></span>
					<div class="row">${this.renderNextButton()}</div>
				</div>
			`;
		} else if (this.currentPage === 1) {
			return html`
				<div class="column" style="gap: 16px; flex: 1">
					<span class="smaller-title">${msg('You own your data.')} </span>
					<span>${msg('Your messages are stored in your device.')} </span>

					<span style="flex: 1"></span>

					${this.renderButtons()}
				</div>
			`;
		} else if (this.currentPage === 2) {
			return html`
				<div class="column" style="gap: 16px; flex: 1">
					<span class="smaller-title">${msg('Preserve your privacy.')} </span>
					<span>${msg('All messages are end-to-end encrypted.')} </span>

					<span style="flex: 1"></span>

					${this.renderButtons()}
				</div>
			`;
		} else if (this.currentPage === 3) {
			return html`
				<div class="column" style="gap: 16px; flex: 1">
					<span class="smaller-title">${msg('Chat even when offline.')} </span>
					<span
						>${msg(
							"Dash chat works as long as you can connect to your friends' devices.",
						)}
					</span>

					<span style="flex: 1"></span>

					${this.renderButtons()}
				</div>
			`;
		} else if (this.currentPage === 4) {
			return html`
				<div class="column" style="gap: 16px; flex: 1">
					<span class="smaller-title"
						>${msg('Dash Chat is in pre-alpha.')}
					</span>
					<span>${msg('We are still building Dash Chat.')} </span>
					<span
						>${msg(
							'The app is ready for testing and experimentation, but not to be used as a daily driver.',
						)}
					</span>

					<span style="flex: 1"></span>

					${this.renderButtons()}
				</div>
			`;
		} else {
			return html`
				<div class="column" style="gap: 16px; flex: 1">
					<span class="title">${msg("That's it!")} </span>
					<span>${msg('Have fun using dash chat.')} </span>

					<span style="flex: 1"></span>
					${this.initialized
						? html``
						: html`
								<span class="placeholder"
									>${msg('Initializing app... This may take a few seconds.')}
								</span>
							`}

					<sl-button
						size="large"
						variant="primary"
						.disabled=${!this.initialized}
						.loading=${!this.initialized}
						@click=${async (e: CustomEvent) => {
							const button = e.target as SlButton;
							button.disabled = true;
							try {
								if (isMobileOs()) {
									let permissionGranted = await isPermissionGranted();
									if (!permissionGranted) {
										button.loading = true;
										const permission = await requestPermission();
										permissionGranted = permission === 'granted';
									}
								}
							} catch (e) {
								console.error(e);
								notifyError(msg('Failed to setup push notifications.'));
							}
							button.loading = false;
							button.disabled = false;
							completeSplascreen();
							this.dispatchEvent(
								new CustomEvent('start-app-clicked', {
									bubbles: true,
									composed: true,
								}),
							);
						}}
						>${msg('Start app')}</sl-button
					>
				</div>
			`;
		}
	}

	renderDesktop() {
		return html`
			<div
				class="row"
				style="flex: 1; align-items: center; justify-content: center"
			>
				<sl-card style="height: 350px; width: 600px">
					${this.renderPage()}
				</sl-card>
			</div>
		`;
	}
	renderMobile() {
		return html`
			<div class="row" style="flex: 1; margin: 24px">${this.renderPage()}</div>
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
			}
			span,
			li {
				font-size: 24px;
			}
			span.title {
				font-size: 48px;
			}
			span.smaller-title {
				font-size: 32px;
			}
		`,
		appStyles,
	];
}
