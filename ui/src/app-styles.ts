import { sharedStyles } from '@tnesh-stack/elements';
import { css } from 'lit';

export const appStyles = [
	css`
		.top-bar {
			align-items: center;
			background-color: var(--sl-color-primary-500);
			padding: 0 16px;
			height: 64px;
		}

		group-chat::part(top-bar) {
			background-color: unset;
			color: black;
			box-shadow: rgba(149, 157, 165, 0.2) 0px 4px 8px;
		}
	`,
	...sharedStyles,
];
