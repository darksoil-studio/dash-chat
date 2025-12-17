<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/button/button.js';
	import '@awesome.me/webawesome/dist/components/badge/badge.js';
	import '@awesome.me/webawesome/dist/components/avatar/avatar.js';
	import '@awesome.me/webawesome/dist/components/relative-time/relative-time.js';
	import '@awesome.me/webawesome/dist/components/format-date/format-date.js';
	import { ChatsStore } from 'dash-chat-stores';
	import { getContext } from 'svelte';
	import { useReactivePromise } from '../stores/use-signal';

	const chatsStore: ChatsStore = getContext('chats-store');
	const chatSummaries = useReactivePromise(chatsStore.allChatsSummaries);

	const today = new Date();
	today.setHours(0);
	today.setMinutes(0);
	today.setMilliseconds(0);
	const todayFirstTimestamp = today.valueOf();
	const yesterday = new Date();
	yesterday.setDate(today.getDate() - 1);
	yesterday.setHours(0);
	yesterday.setMinutes(0);
	yesterday.setMilliseconds(0);
	const yesterdayFirstTimestamp = yesterday.valueOf();

	const beforeThanYesterday = (timestamp: number) =>
		timestamp < yesterdayFirstTimestamp;

	const inYesterday = (timestamp: number) =>
		yesterdayFirstTimestamp < timestamp && timestamp < todayFirstTimestamp;
	const lessThanAMinuteAgo = (timestamp: number) =>
		Date.now() - timestamp < 60 * 1000;
	const moreThanAnHourAgo = (timestamp: number) =>
		Date.now() - timestamp > 46 * 60 * 1000;
</script>

<div class="column" style="flex: 1;">
	{#await $chatSummaries then summaries}
		{#each summaries as summary}
			<wa-button
				class="button-with-avatar"
				appearance="plain"
				href={`/group-chat/${summary.chatId}`}
			>
				<wa-avatar
					slot="start"
					image={summary.avatar}
					initials={summary.name.slice(0, 2)}
				>
				</wa-avatar>
				<div class="row" style="align-items: center; gap: var(--wa-space-m)">
					<div class="column" style="flex: 1">
						<div
							class="row"
							style="align-items: center; gap: var(--wa-space-s)"
						>
							<span style="flex: 1">{summary.name}</span>

							{#if beforeThanYesterday(summary.lastEvent.timestamp)}
								<wa-format-date
									weekday="short"
									date={new Date(summary.lastEvent.timestamp)}
								></wa-format-date>`;
							{:else if inYesterday(summary.lastEvent.timestamp)}
								yesterday
							{:else if lessThanAMinuteAgo(summary.lastEvent.timestamp)}
								now
							{:else if moreThanAnHourAgo(summary.lastEvent.timestamp)}
								<wa-format-date
									hour="numeric"
									minute="numeric"
									hour-format="24"
									date={new Date(summary.lastEvent.timestamp)}
								></wa-format-date>
							{:else}
								<wa-relative-time
									sync
									style="text-align: right"
									format="narrow"
									date={new Date(summary.lastEvent.timestamp)}
								>
								</wa-relative-time>
							{/if}
						</div>

						<div
							class="row"
							style="align-items: center; gap: var(--wa-space-s)"
						>
							<span style="flex: 1">{summary.lastEvent.summary}</span>

							{#if summary.unreadMessages !== 0}
								<wa-badge variant="brand" pill
									>{summary.unreadMessages}
								</wa-badge>
							{/if}
						</div>
					</div>
				</div>
			</wa-button>
		{:else}
			<span>You don't have any chats yet. </span>
		{/each}
	{/await}
</div>

<style>
	wa-button::part(label) {
		flex: 1;
	}
</style>
