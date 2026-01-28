<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import { m } from '$lib/paraglide/messages.js';
	import { wrapPathInSvg } from '$lib/utils/icon';
	import { mdiSend, mdiEmoticonHappyOutline } from '@mdi/js';
	import { useTheme } from 'konsta/svelte';

	interface Props {
		value?: string;
		placeholder?: string;
		onSend?: () => void;
		onEmojiClick?: () => void;
	}

	let {
		value = $bindable(''),
		placeholder = m.typeMessage(),
		onSend,
		onEmojiClick,
	}: Props = $props();

	const theme = $derived(useTheme());

	let hasText = $derived(value.trim().length > 0);

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' && !event.shiftKey) {
			event.preventDefault();
			if (hasText) {
				onSend?.();
			}
		}
	}

	function handleInput(event: Event) {
		const target = event.target as HTMLTextAreaElement;
		value = target.value;
		autoResize(target);
	}

	function autoResize(textarea: HTMLTextAreaElement) {
		textarea.style.height = 'auto';
		textarea.style.height = Math.min(textarea.scrollHeight, 100) + 'px';
	}

	function handleSendClick() {
		if (hasText) {
			onSend?.();
		}
	}
</script>

<div class="message-input-bar" class:ios={theme === 'ios'}>
	<div class="input-row">
		<div class="input-container bg-white dark:bg-gray-400">
			{#if onEmojiClick}
				<button
					type="button"
					class="icon-button emoji-btn"
					onclick={onEmojiClick}
					aria-label="Emoji"
				>
					<wa-icon src={wrapPathInSvg(mdiEmoticonHappyOutline)}></wa-icon>
				</button>
			{/if}

			<textarea
				class="message-textarea"
				{placeholder}
				bind:value
				rows="1"
				onkeydown={handleKeydown}
				oninput={handleInput}
			></textarea>
		</div>

		<button
			type="button"
			class="send-button"
			class:active={hasText}
			onclick={handleSendClick}
			disabled={!hasText}
			aria-label="Send"
		>
			<wa-icon src={wrapPathInSvg(mdiSend)}></wa-icon>
		</button>
	</div>
</div>

<style>
	.message-input-bar {
		position: fixed;
		bottom: 0;
		left: 0;
		right: 0;
		background: var(--k-bars-bg-color);
		padding: 8px 12px;
		z-index: 100;
	}

	.message-input-bar.ios {
		padding-bottom: max(8px, env(safe-area-inset-bottom));
	}

	.input-row {
		display: flex;
		align-items: flex-end;
		gap: 8px;
		max-width: 680px;
		margin: 0 auto;
	}

	.input-container {
		flex: 1;
		display: flex;
		align-items: flex-end;
		min-width: 0;
		border: 1px solid var(--k-hairline-color);
		border-radius: 22px;
		padding: 4px 4px 4px 6px;
		transition: border-color 0.15s ease;
	}

	.input-container:focus-within {
		border-color: var(--k-theme-color, #3b82f6);
	}

	.icon-button {
		flex-shrink: 0;
		width: 36px;
		height: 36px;
		border: none;
		background: transparent;
		border-radius: 50%;
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		color: var(--k-text-color);
		opacity: 0.5;
		transition:
			opacity 0.15s ease,
			background-color 0.15s ease;
		padding: 0;
	}

	.icon-button:hover {
		opacity: 0.8;
		background: rgba(128, 128, 128, 0.1);
	}

	.icon-button:active {
		background: rgba(128, 128, 128, 0.2);
	}

	.message-textarea {
		flex: 1;
		min-width: 0;
		border: none;
		outline: none;
		resize: none;
		font-size: 16px;
		line-height: 1.375;
		padding: 8px 8px;
		color: var(--k-text-color);
		font-family: inherit;
		min-height: 20px;
		max-height: 100px;
		overflow-y: auto;
	}

	.message-textarea::placeholder {
		color: var(--k-list-input-placeholder-color);
		opacity: 0.6;
	}

	.send-button {
		flex-shrink: 0;
		width: 40px;
		height: 40px;
		border: none;
		border-radius: 50%;
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		padding: 0;
		background: rgba(128, 128, 128, 0.15);
		color: var(--k-text-color);
		opacity: 0.4;
		transition:
			background-color 0.2s ease,
			opacity 0.2s ease,
			transform 0.1s ease;
	}

	.send-button:disabled {
		cursor: default;
	}

	.send-button.active {
		background: var(--k-theme-color, #3b82f6);
		color: white;
		opacity: 1;
	}

	.send-button.active:hover {
		filter: brightness(1.1);
	}

	.send-button.active:active {
		transform: scale(0.95);
	}

	/* Icon sizing */
	.icon-button :global(wa-icon),
	.send-button :global(wa-icon) {
		width: 22px;
		height: 22px;
	}

	.send-button :global(wa-icon) {
		margin-left: 2px; /* Optical centering for send arrow */
	}
</style>
