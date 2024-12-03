import '@darksoil-studio/file-storage-zome/dist/elements/show-avatar-image.js';
import {
	MessengerStore,
	messengerStoreContext,
} from '@darksoil-studio/messenger-zome';
import '@darksoil-studio/messenger-zome/dist/elements/all-chats.js';
import '@darksoil-studio/messenger-zome/dist/elements/group-chat.js';
import '@darksoil-studio/messenger-zome/dist/elements/group-details.js';
import '@darksoil-studio/messenger-zome/dist/elements/peer-chat.js';
import {
	ProfilesStore,
	profilesStoreContext,
} from '@darksoil-studio/profiles-zome';
import '@darksoil-studio/profiles-zome/dist/elements/all-profiles.js';
import '@darksoil-studio/profiles-zome/dist/elements/profile-list-item.js';
import {
	AgentPubKey,
	AppClient,
	EntryHash,
	decodeHashFromBase64,
	encodeHashToBase64,
} from '@holochain/client';
import { consume } from '@lit/context';
import { msg } from '@lit/localize';
import {
	mdiAccount,
	mdiAccountGroup,
	mdiAccountMultiple,
	mdiAccountMultiplePlus,
	mdiArrowLeft,
	mdiChat,
	mdiChatOutline,
	mdiDotsVertical,
} from '@mdi/js';
import '@shoelace-style/shoelace/dist/components/divider/divider.js';
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
	wrapPathInSvg,
} from '@tnesh-stack/elements';
import '@tnesh-stack/elements/dist/elements/display-error.js';
import { AsyncResult, SignalWatcher, toPromise } from '@tnesh-stack/signals';
import { EntryRecord, encodeAppEntry } from '@tnesh-stack/utils';
import { LitElement, css, html } from 'lit';
import { customElement } from 'lit/decorators.js';

import { appStyles } from './app-styles.js';
import { isMobileContext } from './context.js';

@customElement('home-page')
export class HomePage extends SignalWatcher(LitElement) {
	@consume({ context: appClientContext })
	client!: AppClient;

	@consume({ context: profilesStoreContext, subscribe: true })
	profilesStore!: ProfilesStore;

	@consume({ context: messengerStoreContext, subscribe: true })
	messengerStore!: MessengerStore;

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
					style="flex: 1; margin: 8px"
				></peer-chat>`,
		},
		{
			path: 'peer/:peer',
			render: ({ peer }) =>
				html`<peer-chat
					.peer=${decodeHashFromBase64(peer!)}
					style="flex: 1; margin: 8px"
				></peer-chat>`,
		},
		{
			path: 'group-chat/:groupChatHash',
			render: ({ groupChatHash }) =>
				html`<group-chat
					.groupChatHash=${decodeHashFromBase64(groupChatHash!)}
					style="flex: 1; margin: 8px"
				>
					<sl-icon-button
						style="color: black"
						slot="top-bar-left-action"
						.src=${wrapPathInSvg(mdiArrowLeft)}
						@click=${() => this.routes.goto(`group-chat/${groupChatHash}`)}
					></sl-icon-button>
				</group-chat>`,
		},
		{
			path: 'group-chat/:groupChatHash/details',
			render: ({ groupChatHash }) => html`
				<group-details
					.groupChatHash=${decodeHashFromBase64(groupChatHash!)}
					style="flex: 1; margin: 8px"
				></group-details>
			`,
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
			<sl-tab-group placement="bottom" style="flex: 1; margin: 0 8px">
				<sl-tab style="flex: 1" slot="nav" panel="all_chats">
					<div class="column" style="align-items: center; gap: 8px; flex: 1">
						<sl-icon
							.src=${wrapPathInSvg(mdiChat)}
							style="font-size: 24px"
						></sl-icon>
						<span> ${msg('Chats')} </span>
					</div>
				</sl-tab>
				<sl-tab style="flex: 1" slot="nav" panel="all_profiles">
					<div class="column" style="align-items: center; gap: 8px; flex: 1">
						<sl-icon
							.src=${wrapPathInSvg(mdiAccountMultiple)}
							style="font-size: 24px"
						></sl-icon>
						<span> ${msg('Members')} </span>
					</div></sl-tab
				>
				<sl-tab-panel name="all_chats">
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
				</sl-tab-panel>
				<sl-tab-panel name="all_profiles">
					<all-profiles
						.excludedProfiles=${this.myProfile.status === 'completed'
							? [this.myProfile.value?.profileHash]
							: []}
						style="flex: 1; margin: 8px"
						@profile-selected=${async (e: CustomEvent) => {
							const agents = await toPromise(
								this.profilesStore.agentsForProfile.get(e.detail.profileHash),
							);
							this.routes.goto(`peer/${encodeHashToBase64(agents[0])}`);
						}}
					>
					</all-profiles>
				</sl-tab-panel>
			</sl-tab-group>
		`;
	}
	get myProfile() {
		return this.profilesStore.myProfile.get();
	}

	@consume({ context: isMobileContext })
	isMobile!: boolean;

	renderContent() {
		if (this.isMobile) {
			if (this.routes.currentPathname() === '') {
				return this.renderHomePanel();
			} else {
				return this.routes.outlet();
			}
		}

		return html`
			<div class="row" style="flex: 1">
				<div class="column" style="flex-basis: 400px">
					${this.renderHomePanel()}
				</div>

				<sl-divider vertical> </sl-divider>

				${this.routes.outlet()}
			</div>
		`;
	}

	renderActions() {
		if (this.isMobile) {
			return html`
				<sl-dropdown>
					<sl-icon-button
						slot="trigger"
						style="font-size: 24px; color: var(--sl-color-neutral-900)"
						.src=${wrapPathInSvg(mdiDotsVertical)}
					></sl-icon-button>
					<sl-menu
						@sl-select=${(e: CustomEvent) => {
							const item = e.detail.item as SlMenuItem;
							const value = item.value;
							if (value === 'new_group') {
								this.dispatchEvent(
									new CustomEvent('create-group-chat-selected'),
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
			<div class="row" style="gap: 16px; align-items: center">
				<sl-button
					style="font-size: 24px"
					@click=${() =>
						this.dispatchEvent(new CustomEvent('create-group-chat-selected'))}
					><sl-icon
						slot="prefix"
						.src=${wrapPathInSvg(mdiAccountMultiplePlus)}
					></sl-icon
					>${msg('Create Group')}
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

	peerChatNickname(peerChatHash: EntryHash): AsyncResult<string> {
		const peerChat = this.messengerStore.peerChats
			.get(peerChatHash)
			.currentPeerChat.get();
		if (peerChat.status !== 'completed') return peerChat;

		const peer = peerChat.value.peer_1.agents.find(
			a =>
				encodeHashToBase64(a) ===
				encodeHashToBase64(this.profilesStore.client.client.myPubKey),
		)
			? peerChat.value.peer_2
			: peerChat.value.peer_1;

		if (peer.profile) {
			return {
				status: 'completed',
				value: peer.profile.nickname,
			};
		}
		return this.agentNickname(peer.agents[0]);
	}
	agentNickname(agent: AgentPubKey): AsyncResult<string> {
		const profile = this.profilesStore.profileForAgent.get(agent).get();
		if (profile.status !== 'completed') return profile;
		const latestVersion = profile.value!.latestVersion.get();
		if (latestVersion.status !== 'completed') return latestVersion;

		return {
			status: 'completed',
			value: latestVersion.value.entry.nickname,
		};
	}

	renderChatName() {
		const params = this.routes.params;
		const isPeerChat = this.routes.currentPathname().startsWith('peer-chat');
		if (isPeerChat) {
			const peerChatHash = decodeHashFromBase64(params.peerChatHash!);
			const nickname = this.peerChatNickname(peerChatHash);
			if (nickname.status !== 'completed') return html``;
			return html`<span>${nickname.value}</span>`;
		}
		const isGroupChat = this.routes.currentPathname().startsWith('group-chat');
		if (isGroupChat) {
			const groupChatHash = decodeHashFromBase64(params.groupChatHash!);
			const groupChat = this.messengerStore.groupChats
				.get(groupChatHash)
				.currentGroupChat.get();
			if (groupChat.status !== 'completed') return html``;

			return html`
				<div
					class="row"
					style="flex: 1; align-items: center; gap: 8px; cursor: pointer"
					@click=${() => {
						this.routes.goto(
							`group-chat/${encodeHashToBase64(groupChatHash)}/details`,
						);
					}}
				>
					<show-avatar-image .imageHash=${groupChat.value.info.avatar_hash}>
					</show-avatar-image>
					<span>${groupChat.value.info.name} </span>
				</div>
			`;
		}
		const isPeer = this.routes.currentPathname().startsWith('peer');
		if (isPeer) {
			const peer = decodeHashFromBase64(params.peer!);
			const nickname = this.agentNickname(peer);
			if (nickname.status !== 'completed') return html``;
			return html`<span>${nickname.value}</span>`;
		}
	}

	renderTopBar() {
		if (this.isMobile && this.routes.currentPathname() !== '') {
			return html`
				<div class="row top-bar" style="gap: 8px">
					<sl-icon-button
						style="color: black"
						.src=${wrapPathInSvg(mdiArrowLeft)}
						@click=${() => this.routes.goto('')}
					></sl-icon-button>
					${this.renderChatName()}
				</div>
			`;
		}

		return html`
			<div class="row top-bar">
				<span class="title" style="flex: 1">${msg('Messenger Demo')}</span>

				${this.renderActions()}
			</div>
		`;
	}

	render() {
		return html`
			<div class="column" style="flex: 1">
				${this.renderTopBar()} ${this.renderContent()}
			</div>
		`;
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
		`,
		...appStyles,
	];
}
