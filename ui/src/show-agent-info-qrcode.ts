import { AdminWebsocket, AgentInfoSigned } from '@holochain/client';
import { consume } from '@lit/context';
import { localized } from '@lit/localize';
import { decode, encode } from '@msgpack/msgpack';
import '@shoelace-style/shoelace/dist/components/dialog/dialog.js';
import '@shoelace-style/shoelace/dist/components/qr-code/qr-code.js';
import { Format, scan } from '@tauri-apps/plugin-barcode-scanner';
import { SignalWatcher } from '@tnesh-stack/signals';
import { fromUint8Array, toUint8Array } from 'js-base64';
import { LitElement, html } from 'lit';
import { customElement } from 'lit/decorators.js';

import { adminWebsocketContext } from './context.js';

export async function scanAgentInfoQrcode(): Promise<Array<AgentInfoSigned>> {
	const result = await scan({ windowed: true, formats: [Format.QRCode] });
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

	agentInfos!: Array<AgentInfoSigned>;

	async firstUpdated() {
		this.agentInfos = await this.adminWebsocket.agentInfo({
			cell_id: null,
		});
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
				style="flex: 1"
			></sl-qr-code>
		`;
	}

	render() {
		return html` <sl-dialog>${this.renderQrcode()} </sl-dialog> `;
	}
}
