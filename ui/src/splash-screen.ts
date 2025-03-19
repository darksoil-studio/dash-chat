import { AppWebsocket } from '@holochain/client';
import { localized, msg } from '@lit/localize';
import '@shoelace-style/shoelace/dist/components/button/button.js';
import { SignalWatcher } from '@tnesh-stack/signals';
import { LitElement, css, html } from 'lit';
import { customElement, state } from 'lit/decorators.js';

import { appStyles } from './app-styles';

@localized()
@customElement('splash-screen')
export class SplashScreen extends SignalWatcher(LitElement) {
	@state()
	initialized = false;

	firstUpdated() {
		this.attemptConnect();
	}

	async attemptConnect() {
		try {
			const _client = await AppWebsocket.connect();
			// await client.callZome({
			// 	role_name: 'main',
			// 	zome_name: 'messenger',
			// 	fn_name: 'init',
			// 	payload: undefined,
			// });
			this.initialized = true;
		} catch (e: unknown) {
			setTimeout(() => this.attemptConnect(), 300);
		}
	}

	render() {
		return html`
			<div class="column" style="flex: 1">
				<img
					src="../splashscreen.jpg"
					style="height: 300px; width: 100%; object-fit: cover"
				/>

				<div class="column" style="gap: 16px; margin: 16px; flex: 1">
					<span class="title">${msg('Welcome to dash chat!')} </span>
					<span>${msg('A private and offline first chat app.')} </span>

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
			</div>
		`;
	}

	static styles = [
		css`
			:host {
				display: flex;
			}
		`,
		appStyles,
	];
}
