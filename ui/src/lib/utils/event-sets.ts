import type { AgentId, Hash, SimplifiedOperation } from 'dash-chat-stores';

export const MESSAGE_SET_TIMEFRAME_INTERVAL = 60 * 1000 * 1000; // 1 minute

export interface EventSetsInDay<T> {
	day: Date;
	eventsSets: Array<EventSet<T>>;
}

export type EventSet<T> = Array<[Hash, SimplifiedOperation<T>]>;

export function orderInEventSets<T>(
	events: Record<Hash, SimplifiedOperation<T>>,
	agentSets: Array<Array<AgentId>>,
): Array<EventSetsInDay<T>> {
	const eventsSetsInDay: EventSetsInDay<T>[] = [];
	const orderedDescendingEvents = Object.entries(events).sort(
		(m1, m2) => m2[1].header.timestamp - m1[1].header.timestamp,
	);
	for (const [eventHash, event] of orderedDescendingEvents) {
		if (eventsSetsInDay.length === 0) {
			const date = new Date(event.header.timestamp * 1000);
			date.setHours(0);
			date.setMinutes(0);
			date.setSeconds(0);
			date.setMilliseconds(0);
			eventsSetsInDay.push({
				eventsSets: [[[eventHash, event]]],
				day: date,
			});
		} else {
			const lastEventSetsInDay = eventsSetsInDay[eventsSetsInDay.length - 1];
			const lastEventSet =
				lastEventSetsInDay.eventsSets[lastEventSetsInDay.eventsSets.length - 1];

			const lastEvent = lastEventSet[lastEventSet.length - 1][1];

			const lastMessageAgentSet = agentSets.find(agents =>
				agents.find(agent => agent === lastEvent.header.public_key),
			);

			const currentMessageAgentSet = agentSets.find(agents =>
				agents.find(agent => agent === event.header.public_key),
			);

			const sameProvenance = lastMessageAgentSet === currentMessageAgentSet;
			const sameTimeframe =
				lastEvent.header.timestamp - event.header.timestamp <
				MESSAGE_SET_TIMEFRAME_INTERVAL;

			const date = new Date(event.header.timestamp * 1000);
			date.setHours(0);
			date.setMinutes(0);
			date.setSeconds(0);
			date.setMilliseconds(0);

			if (date.valueOf() === lastEventSetsInDay.day.valueOf()) {
				if (sameProvenance && sameTimeframe) {
					lastEventSet.push([eventHash, event]);
				} else {
					lastEventSetsInDay.eventsSets.push([[eventHash, event]]);
				}
			} else {
				eventsSetsInDay.push({
					eventsSets: [[[eventHash, event]]],
					day: date,
				});
			}
		}
	}

	return eventsSetsInDay;
}
