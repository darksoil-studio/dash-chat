import { AdminWebsocket, AgentInfoSigned } from '@holochain/client';
import { consume } from '@lit/context';
import { localized } from '@lit/localize';
import { decode, encode } from '@msgpack/msgpack';
import '@shoelace-style/shoelace/dist/components/dialog/dialog.js';
import '@shoelace-style/shoelace/dist/components/qr-code/qr-code.js';
import {
	Format,
	requestPermissions,
	scan,
} from '@tauri-apps/plugin-barcode-scanner';
import { SignalWatcher } from '@tnesh-stack/signals';
import { fromUint8Array, toUint8Array } from 'js-base64';
import { LitElement, html } from 'lit';
import { customElement, state } from 'lit/decorators.js';

import { adminWebsocketContext } from './context.js';

export async function scanAgentInfoQrcode(): Promise<Array<AgentInfoSigned>> {
	await requestPermissions();
	const result = await scan({ windowed: false, formats: [Format.QRCode] });
	const agentInfos = decode(
		toUint8Array(result.content),
	) as Array<AgentInfoSigned>;

	return agentInfos;
}

@localized()
@customElement('show-agent-info-qrcode')
export class ShowAgentInfoQrcode extends SignalWatcher(LitElement) {
	@consume({ context: adminWebsocketContext, subscribe: true })
	adminWebsocket!: AdminWebsocket;

	@state()
	agentInfos!: Array<AgentInfoSigned>;

	async firstUpdated() {
		console.log('a', this.adminWebsocket);
		this.agentInfos = await this.adminWebsocket.agentInfo({
			cell_id: null,
		});
		console.log('aaa', this.agentInfos);
	}

	get dialog() {
		return this.shadowRoot!.querySelector('sl-dialog')!;
	}

	show() {
		this.dialog.show();
	}

	renderQrcode() {
		if (!this.agentInfos) return html``;

		return html`
			<sl-qr-code
				value="${fromUint8Array(encode(this.agentInfos))}"
				size="500"
			></sl-qr-code>
		`;
	}

	render() {
		return html`
			<sl-dialog style="--width: 35.2rem">${this.renderQrcode()} </sl-dialog>
		`;
	}
}
