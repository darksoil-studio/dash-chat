import { wrapPathInSvg } from '@darksoil-studio/holochain-elements';
import { SignalWatcher } from '@darksoil-studio/holochain-signals';
import { consume } from '@lit/context';
import { mdiArrowLeft, mdiClose } from '@mdi/js';
import { LitElement, TemplateResult, css, html, render } from 'lit';
import { customElement, property } from 'lit/decorators.js';
import { styleMap } from 'lit/directives/style-map.js';

import { appStyles } from './app-styles.js';
import { isMobileContext } from './context.js';

@customElement('overlay-page')
export class OverlayPage extends SignalWatcher(LitElement) {
	@property()
	title: string = '';

	@property()
	icon: 'back' | 'close' = 'close';

	@property()
	@consume({ context: isMobileContext })
	isMobile: boolean = false;

	mdiIcon() {
		if (this.icon === 'back') return mdiArrowLeft;
		return mdiClose;
	}

	render() {
		return html`
			<div class="column fill">
				<div class="row top-bar" style="gap: 8px">
					<sl-icon-button
						@click=${() =>
							this.dispatchEvent(new CustomEvent('close-requested'))}
						.src=${wrapPathInSvg(this.mdiIcon())}
					></sl-icon-button>
					<span class="title" style="flex: 1">${this.title}</span>

					<slot name="action"></slot>
				</div>

				<div class="flex-scrollable-parent" style="flex:1">
					<div class="flex-scrollable-container" style="flex:1">
						<div class="flex-scrollable-y" style="height: 100%;">
							<div
								class="column"
								style="min-height: calc(100% - 32px); margin: 16px; align-items: center"
							>
								<div
									class="column"
									style=${styleMap({
										'min-width': this.isMobile ? '100%' : '500px',
										flex: '1',
									})}
								>
									<slot></slot>
								</div>
							</div>
						</div>
					</div>
				</div>
			</div>
		`;
	}

	static styles = [
		...appStyles,
		css`
			:host {
				position: fixed;
				top: 0;
				bottom: 0;
				left: 0;
				right: 0;
				z-index: 799;
				background: var(--background-color);
			}
		`,
	];
}
