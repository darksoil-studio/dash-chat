<script lang="ts">
	import { AsyncRelay } from '../signals/relay';

	const clock = new AsyncRelay<number>(async (set, get) => {
	console.log('maker')
		const i = setInterval(() => {
			const v = get();
			const c = v.status === 'completed' ? v.value : 0;
			set(c + 1);
		}, 1000);
		return () => clearInterval(i);
	});

	import '@awesome.me/webawesome/dist/components/button/button.js';
	import '@awesome.me/webawesome/dist/components/select/select.js';
	import '@awesome.me/webawesome/dist/components/button-group/button-group.js';
	import '@awesome.me/webawesome/dist/components/switch/switch.js';
	import '@awesome.me/webawesome/dist/components/input/input.js';
	import { invoke } from '@tauri-apps/api/core';

	let name = $state('');
	let greetMsg = $state('');

	async function greet(event: Event) {
		event.preventDefault();
		// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
		greetMsg = await invoke('greet', { name });
	}
</script>

<main class="container">
	{$clock.value}
	<wa-button variant="brand" onclick={() => console.log('aaa')}>aaa</wa-button>
	<wa-button-group label="Alignment">
		<wa-button>Left</wa-button>
		<wa-button>Center</wa-button>
		<wa-button>Right</wa-button>
	</wa-button-group>
	<wa-select>
		<wa-option value="">Option 1</wa-option>
		<wa-option value="option-2">Option 2</wa-option>
		<wa-option value="option-3">Option 3</wa-option>
		<wa-option value="option-4">Option 4</wa-option>
		<wa-option value="option-5">Option 5</wa-option>
		<wa-option value="option-6">Option 6</wa-option>
	</wa-select>

	<wa-switch> </wa-switch>
	<wa-input> </wa-input>
</main>

<style>
	@media (prefers-color-scheme: dark) {
		:root {
			color: #f6f6f6;
			background-color: #2f2f2f;
		}

		a:hover {
			color: #24c8db;
		}
	}
</style>
