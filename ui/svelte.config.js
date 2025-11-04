// Tauri doesn't have a Node.js server to do proper SSR
// so we use adapter-static with a fallback to index.html to put the site in SPA mode
// See: https://svelte.dev/docs/kit/single-page-apps
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
import adapter from '@sveltejs/adapter-static';
import { signaliumPreset } from 'signalium/transform';
import { sveltePreprocess } from 'svelte-preprocess';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	preprocess: [
		sveltePreprocess({
			babel: {
				presets: [
					// [
					// 	'@babel/preset-env',
					// 	{
					// 		loose: true,
					// 		modules: false,
					// 		targets: {
					// 			esmodules: true,
					// 		},
					// 	},
					// ],
					signaliumPreset(),
				],
				// plugins: [],
			},
		}),
		{
			name: 'a',
			markup: ({ content }) => {
				console.log('hey', content);
			},
			script: ({ content }) => console.log('hey', content),
		},
	],

	kit: {
		adapter: adapter({
			fallback: 'index.html',
		}),
	},
};

export default config;
