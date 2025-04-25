import { sharedStyles } from '@darksoil-studio/holochain-elements';
import { css } from 'lit';

export const appStyles = [
	css`
		.top-bar {
			align-items: center;
			padding: 0 16px;
			height: 64px;
			background-color: var(--sl-color-neutral-100);
			border: 1px solid var(--sl-color-gray-300, lightgrey);
			box-shadow: rgba(149, 157, 165, 0.2) 2px 2px 4px;
		}
		group-chat::part(top-bar) {
			height: 42px;
		}
		peer-chat::part(top-bar) {
			height: 42px;
		}
	`,
	...sharedStyles,
];
