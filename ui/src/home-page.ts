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
import { msg, str } from '@lit/localize';
import {
	mdiAccount,
	mdiAccountMultiple,
	mdiAccountMultiplePlus,
	mdiArrowLeft,
	mdiChat,
	mdiChatOutline,
	mdiDotsVertical,
	mdiMessagePlus,
} from '@mdi/js';
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
import { AsyncResult, SignalWatcher, toPromise } from '@tnesh-stack/signals';
import { EntryRecord, encodeAppEntry } from '@tnesh-stack/utils';
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
									style="color: black"
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
									style="color: black"
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
									style="color: black"
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
			class="column"
			style="flex: 1; align-items: center; justify-content: center; gap: 24px"
		>
			<sl-icon .src=${wrapPathInSvg(mdiChatOutline)} style="font-size: 48px">
			</sl-icon>
			<span>${msg('Select a chat.')}</span>
		</div>`;
	}

	renderHomePanel() {
		return html`
			<all-chats
				style="flex: 1; margin: 8px"
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
				<show-agent-info-qrcode> </show-agent-info-qrcode>
				<sl-dropdown>
					<sl-icon-button
						slot="trigger"
						style="font-size: 24px; color: var(--sl-color-neutral-900)"
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
			<show-agent-info-qrcode> </show-agent-info-qrcode>
			<div class="row" style="gap: 16px; align-items: center">
				<sl-button
					style="font-size: 24px"
					@click=${() =>
						this.dispatchEvent(new CustomEvent('create-group-chat-selected'))}
					variant="text"
					><sl-icon
						slot="prefix"
						.src=${wrapPathInSvg(mdiAccountMultiplePlus)}
					></sl-icon
					>${msg('New group')}
				</sl-button>
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
		`;
	}

	renderMobile() {
		if (this.routes.currentPathname() !== '') return this.routes.outlet();
		return html`
			<div class="column" style="flex: 1">
				<div class="row top-bar">
					<span class="title" style="flex: 1">${msg('Messenger Demo')}</span>

					${this.renderActions()}
					<sl-button
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
				</div>
				${this.renderHomePanel()}
			</div>
		`;
	}

	renderDesktop() {
		return html`
			<div class="column" style="flex: 1">
				<div class="row top-bar">
					<span class="title" style="flex: 1">${msg('Messenger Demo')}</span>

					${this.renderActions()}
				</div>
				<div class="row" style="flex: 1">
					<div class="column" style="flex-basis: 400px">
						${this.renderHomePanel()}
					</div>

					<sl-divider vertical> </sl-divider>

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
		`,
		...appStyles,
	];
}
