<script lang="ts">
	import type { MessageStatus } from 'dash-chat-stores';

	interface Props {
		status: MessageStatus;
	}

	let { status }: Props = $props();
</script>

<span class="message-status ml-1 inline-flex items-center">
	{#if status === 'read'}
		<!-- Two overlapping filled bubbles with checkmark (read) -->
		<svg viewBox="0 0 24 16" class="w-5 h-3.5">
			<!-- Back bubble (filled) -->
			<circle cx="9" cy="8" r="7" fill="currentColor" />
			<!-- Front bubble (filled) -->
			<circle cx="15" cy="8" r="7" fill="currentColor" />
			<!-- Checkmark (contrasting color) -->
			<path
				d="M8 8l3 3 6-6"
				fill="none"
				stroke="var(--message-status-check, white)"
				stroke-width="2"
				stroke-linecap="round"
				stroke-linejoin="round"
			/>
		</svg>
	{:else if status === 'received'}
		<!-- Two overlapping outlined bubbles with checkmark (received) -->
		<svg viewBox="0 0 24 16" class="w-5 h-3.5">
			<!-- Back bubble (outline) -->
			<circle cx="9" cy="8" r="6.5" fill="none" stroke="currentColor" stroke-width="1.2" />
			<!-- Front bubble (outline) -->
			<circle cx="15" cy="8" r="6.5" fill="none" stroke="currentColor" stroke-width="1.2" />
			<!-- Checkmark -->
			<path
				d="M8 8l3 3 6-6"
				fill="none"
				stroke="currentColor"
				stroke-width="1.5"
				stroke-linecap="round"
				stroke-linejoin="round"
			/>
		</svg>
	{:else}
		<!-- Single outlined bubble with checkmark (sent) -->
		<svg viewBox="0 0 16 16" class="w-3.5 h-3.5">
			<!-- Single bubble (outline) -->
			<circle cx="8" cy="8" r="6.5" fill="none" stroke="currentColor" stroke-width="1.2" />
			<!-- Checkmark -->
			<path
				d="M5 8l2 2 4-4"
				fill="none"
				stroke="currentColor"
				stroke-width="1.5"
				stroke-linecap="round"
				stroke-linejoin="round"
			/>
		</svg>
	{/if}
</span>
