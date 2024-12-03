import '@darksoil-studio/messenger-zome/dist/elements/all-chats.js';
import '@darksoil-studio/messenger-zome/dist/elements/group-chat.js';
import '@darksoil-studio/messenger-zome/dist/elements/peer-chat.js';
import {
	ProfilesStore,
	profilesStoreContext,
} from '@darksoil-studio/profiles-zome';
import '@darksoil-studio/profiles-zome/dist/elements/all-profiles.js';
import '@darksoil-studio/profiles-zome/dist/elements/profile-list-item.js';
import {
	AppClient,
	decodeHashFromBase64,
	encodeHashToBase64,
} from '@holochain/client';
import { consume } from '@lit/context';
import { msg } from '@lit/localize';
import { mdiChat } from '@mdi/js';
import '@shoelace-style/shoelace/dist/components/divider/divider.js';
import '@shoelace-style/shoelace/dist/components/icon/icon.js';
import '@shoelace-style/shoelace/dist/components/tab-group/tab-group.js';
import '@shoelace-style/shoelace/dist/components/tab-panel/tab-panel.js';
import '@shoelace-style/shoelace/dist/components/tab/tab.js';
import {
	Router,
	Routes,
	appClientContext,
	wrapPathInSvg,
} from '@tnesh-stack/elements';
import '@tnesh-stack/elements/dist/elements/display-error.js';
import { AsyncResult, SignalWatcher, toPromise } from '@tnesh-stack/signals';
import { EntryRecord, encodeAppEntry } from '@tnesh-stack/utils';
import { LitElement, css, html } from 'lit';
import { customElement } from 'lit/decorators.js';

import { appStyles } from './app-styles.js';

@customElement('home-page')
export class HomePage extends SignalWatcher(LitElement) {
	@consume({ context: appClientContext })
	client!: AppClient;

	@consume({ context: profilesStoreContext, subscribe: true })
	profilesStore!: ProfilesStore;

	routes = new Routes(this, [
		{
			path: '/',
			render: () => this.renderPlaceholder(),
		},
		{
			path: '/peer-chat/:peerChatHash',
			render: ({ peerChatHash }) =>
				html`<peer-chat
					.peerChatHash=${decodeHashFromBase64(peerChatHash!)}
					style="flex: 1"
				></peer-chat>`,
		},
		{
			path: '/peer/:peer',
			render: ({ peer }) =>
				html`<peer-chat
					.peer=${decodeHashFromBase64(peer!)}
					style="flex: 1"
				></peer-chat>`,
		},
		{
			path: '/group-chat/:groupChatHash',
			render: ({ groupChatHash }) =>
				html`<group-chat
					.groupChatHash=${decodeHashFromBase64(groupChatHash!)}
					style="flex: 1"
				></group-chat>`,
		},
	]);

	renderPlaceholder() {
		return html`<div
			class="column"
			style="flex: 1; align-items: center; justify-content: center; gap: 24px"
		>
			<sl-icon .src=${wrapPathInSvg(mdiChat)} style="font-size: 48px">
			</sl-icon>
			<span>${msg('Select a chat')}</span>
		</div>`;
	}

	renderContent() {
		return html`
			<div class="row" style="flex: 1">
				<sl-tab-group>
					<sl-tab slot="nav" panel="all_chats">${msg('Chats')}</sl-tab>
					<sl-tab slot="nav" panel="all_profiles">${msg('Members')}</sl-tab>
					<sl-tab-panel name="all_chats">
						<all-chats
							style="flex: 1"
							@group-chat-selected=${(e: CustomEvent) => {
								this.routes.goto(
									`/group-chat/${encodeHashToBase64(e.detail.groupChatHash)}`,
								);
							}}
							@peer-chat-selected=${(e: CustomEvent) => {
								this.routes.goto(
									`/peer-chat/${encodeHashToBase64(e.detail.peerChatHash)}`,
								);
							}}
						>
						</all-chats>
					</sl-tab-panel>
					<sl-tab-panel name="all_profiles">
						<all-profiles
							style="flex: 1"
							@profile-selected=${async (e: CustomEvent) => {
								const agents = await toPromise(
									this.profilesStore.agentsForProfile.get(e.detail.profileHash),
								);
								this.routes.goto(`/peer/${encodeHashToBase64(agents[0])}`);
							}}
						>
						</all-profiles>
					</sl-tab-panel>
				</sl-tab-group>

				<sl-divider> </sl-divider>

				${this.routes.outlet()}
			</div>
		`;
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
