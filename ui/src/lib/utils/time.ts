export const lessThanAMinuteAgo = (timestamp: number) =>
	Date.now() - timestamp < 60 * 1000;
export const moreThanAnHourAgo = (timestamp: number) =>
	Date.now() - timestamp > 60 * 60 * 1000;
export const moreThanAWeekAgo = (timestamp: number) =>
	Date.now() - timestamp > 7 * 24 * 60 * 60 * 1000;
export const moreThanAYearAgo = (timestamp: number) =>
	Date.now() - timestamp > 365 * 24 * 60 * 60 * 1000;

export const sleep = (ms: number) =>
	new Promise(resolve => setTimeout(() => resolve(undefined), ms));
