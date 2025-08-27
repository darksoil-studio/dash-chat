import { sharedStyles } from '@darksoil-studio/holochain-elements';
import { css } from 'lit';

export const appStyles = [
	css`
		.top-bar {
			padding-top: var(--safe-area-inset-top);
			align-items: center;
			padding-left: 16px;
			padding-right: 16px;
			height: 64px;
			background-color: var(--sl-color-neutral-100);
			border: 1px solid var(--sl-color-gray-300, lightgrey);
			box-shadow: rgba(149, 157, 165, 0.2) 2px 2px 4px;
		}
		group-chat::part(top-bar) {
			padding-top: var(--safe-area-inset-top);
			height: 48px;
			border: 1px solid var(--sl-color-gray-300, lightgrey);
			box-shadow: rgba(149, 157, 165, 0.2) 2px 2px 4px;
		}
		peer-chat::part(top-bar) {
			padding-top: var(--safe-area-inset-top);
			height: 48px;
			border: 1px solid var(--sl-color-gray-300, lightgrey);
			box-shadow: rgba(149, 157, 165, 0.2) 2px 2px 4px;
		}
		sl-button.no-border::part(base) {
			border: none;
		}
	`,
	...sharedStyles,
];
