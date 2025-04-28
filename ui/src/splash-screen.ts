import { SignalWatcher } from '@darksoil-studio/holochain-signals';
import { AppWebsocket } from '@holochain/client';
import { localized, msg } from '@lit/localize';
import '@shoelace-style/shoelace/dist/components/button/button.js';
import '@shoelace-style/shoelace/dist/components/carousel-item/carousel-item.js';
import '@shoelace-style/shoelace/dist/components/carousel/carousel.js';
import { LitElement, css, html } from 'lit';
import { customElement, state } from 'lit/decorators.js';

// @ts-ignore
import imgUrl from '../splashscreen.jpg';
import { appStyles } from './app-styles';

@localized()
@customElement('splash-screen')
export class SplashScreen extends SignalWatcher(LitElement) {
	@state()
	initialized = false;

	@state()
	currentPage = 0;

	firstUpdated() {
		this.attemptConnect();
	}

	async attemptConnect() {
		try {
			const client = await AppWebsocket.connect();
			client
				.callZome({
					role_name: 'main',
					zome_name: 'messenger',
					fn_name: 'init',
					payload: undefined,
				})
				.catch(() =>
					client.callZome({
						role_name: 'main',
						zome_name: 'messenger',
						fn_name: 'init',
						payload: undefined,
					}),
				);
			this.initialized = true;
		} catch (e: unknown) {
			setTimeout(() => this.attemptConnect(), 300);
		}
	}

	renderBackButton() {
		return html`
			<sl-button style="flex: 1" @click=${() => this.currentPage--}
				>${msg('Back')}</sl-button
			>
		`;
	}

	renderNextButton() {
		return html`
			<sl-button
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
						variant="primary"
						.disabled=${!this.initialized}
						.loading=${!this.initialized}
						@click=${() =>
							this.dispatchEvent(
								new CustomEvent('start-app-clicked', {
									bubbles: true,
									composed: true,
								}),
							)}
						>${msg('Start app')}</sl-button
					>
				</div>
			`;
		}
	}

	render() {
		return html`
			<div class="column" style="flex: 1">
				<img
					src="${imgUrl}"
					style="height: 300px; width: 100%; object-fit: cover"
				/>
				<div class="column" style="margin: 24px; flex: 1">
					${this.renderPage()}
				</div>
			</div>
		`;
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
