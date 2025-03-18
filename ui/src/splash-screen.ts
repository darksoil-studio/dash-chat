import { localized, msg } from '@lit/localize';
import '@shoelace-style/shoelace/dist/components/carousel-item/carousel-item.js';
import '@shoelace-style/shoelace/dist/components/carousel/carousel.js';
import { SignalWatcher } from '@tnesh-stack/signals';
import { LitElement, css, html } from 'lit';
import { customElement } from 'lit/decorators.js';

import { appStyles } from './app-styles';

@localized()
@customElement('splash-screen')
export class SplashScreen extends SignalWatcher(LitElement) {
	render() {
		return html`
			<div class="column">
				<sl-carousel>
					<sl-carousel-item>
						<div class="column" style="gap: 16px">
							<span class="title">${msg('Welcome to dash chat')} </span>
							<span>${msg('A private and offline first chat app.')} </span>
						</div>
					</sl-carousel-item>
					<sl-carousel-item>
						<div class="column" style="gap: 16px">
							<span class="title"
								>${msg('Messages are stored only one your devices.')}
							</span>
							<span>${msg('A private and offline first chat app.')} </span>
						</div>
					</sl-carousel-item>
				</sl-carousel>
			</div>
		`;
	}

	static styles = appStyles;
}
