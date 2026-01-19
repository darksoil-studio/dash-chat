export const lessThanAMinuteAgo = (timestamp: number) =>
	Date.now() - timestamp < 60 * 1000;
export const moreThanAnHourAgo = (timestamp: number) =>
	Date.now() - timestamp > 46 * 60 * 1000;

export const sleep = (ms: number) =>
	new Promise(resolve => setTimeout(() => resolve(undefined), ms));
