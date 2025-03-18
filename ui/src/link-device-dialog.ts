import '@darksoil-studio/linked-devices-zome/dist/elements/link-device-recipient.js';
import { consume } from '@lit/context';
import { localized, msg } from '@lit/localize';
import { mdiArrowLeft } from '@mdi/js';
import { SlDialog } from '@shoelace-style/shoelace';
import '@shoelace-style/shoelace/dist/components/dialog/dialog.js';
import { SignalWatcher } from '@tnesh-stack/signals';
import { LitElement, html } from 'lit';
import { customElement, state } from 'lit/decorators.js';
import { styleMap } from 'lit/directives/style-map.js';

import { appStyles } from './app-styles.js';
import { isMobileContext } from './context.js';

@localized()
@customElement('link-device-dialog')
export class LinkDeviceDialog extends SignalWatcher(LitElement) {
	@state()
	linking = false;

	@consume({ context: isMobileContext })
	isMobile!: boolean;

	public show() {
		this.dialog.show();
		this.linking = true;
	}
	get dialog() {
		const dialog = this.shadowRoot!.getElementById('dialog') as SlDialog;
		return dialog;
	}

	render() {
		return html`
			<sl-dialog
				id="dialog"
				.label=${msg('Link Device')}
				@sl-hide=${() => (this.linking = false)}
				style=${styleMap({
					'--width': this.isMobile ? '90vw' : '700px',
				})}
			>
				${this.linking
					? html`
							<link-device-recipient
								@device-linked=${() => {
									this.dialog.hide();
								}}
							>
							</link-device-recipient>
						`
					: html``}
			</sl-dialog>
		`;
	}

	static styles = appStyles;
}
