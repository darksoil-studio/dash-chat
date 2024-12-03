import '@darksoil-studio/profiles-zome/dist/elements/all-profiles.js';
import '@darksoil-studio/profiles-zome/dist/elements/profile-list-item.js';
import { AppClient } from '@holochain/client';
import { consume } from '@lit/context';
import { msg } from '@lit/localize';
import { Router, Routes, appClientContext } from '@tnesh-stack/elements';
import '@tnesh-stack/elements/dist/elements/display-error.js';
import { AsyncResult, SignalWatcher } from '@tnesh-stack/signals';
import { EntryRecord } from '@tnesh-stack/utils';
import { LitElement, css, html } from 'lit';
import { customElement } from 'lit/decorators.js';

import { appStyles } from './app-styles.js';

@customElement('home-page')
export class HomePage extends SignalWatcher(LitElement) {
	@consume({ context: appClientContext })
	client!: AppClient;

	routes = new Routes(this, [
		{
			path: '',
			render: () => html` <all-profiles> </all-profiles> `,
			enter: () => {
				// Redirect to "/home/"
				this.routes.goto('/home/');
				return false;
			},
		},
		{
			path: '/home/*',
			render: () =>
				html`<home-page
					@profile-clicked=${() => this.routes.goto('/my-profile')}
				></home-page>`,
		},
		{
			path: '/my-profile',
			render: () => this.renderMyProfilePage(),
		},
	]);

	renderContent() {
		return html``;
	}

	render() {
		return html`
			<div class="column" style="flex: 1">
				<div class="row top-bar">
					<span class="title" style="flex: 1">${msg('Messenger Demo')}</span>

					<div class="row" style="gap: 16px">
						<profile-list-item
							@click=${() =>
								this.dispatchEvent(
									new CustomEvent('profile-clicked', {
										detail: true,
										composed: true,
									}),
								)}
							.agentPubKey=${this.client.myPubKey}
						></profile-list-item>
					</div>
				</div>

				<div
					class="column"
					style="flex: 1; align-items: center; justify-content: center;"
				>
					${this.renderContent()}
				</div>
			</div>
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
