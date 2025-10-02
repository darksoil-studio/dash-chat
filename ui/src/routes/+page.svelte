<script lang="ts">
	import { AsyncRelay } from '../signals/relay';
	import {listen} from '@tauri-apps/api/event'

	const clock = new AsyncRelay<number>(async (set, get) => {
		const i = setInterval(() => {
			const v = get();
			const c = v.status === 'completed' ? v.value : 0;
			set(c + 1);
		}, 1000);
		return () => clearInterval(i);
	});

	function logStore(topicId: string, author: string) {
		
	return new AsyncRelay<Operation[]>(async (set, get) => {
	'loading'
		const messages = await client.getLog(topicId, author);
		set(messages) // makes it completed
		const unsubs = await listen(event => {
			messages.push(event)
			set(messages)
		})

		return () => {
			unsubs()
		};
	});
	}

	const store =logStore(thistopic, thisauthor)

	async function getMessages() {
		const logs = await getLogs()

		return logs.map(log => {
			
		})
	}

	const messages = new Signal.Computed(()=> {
	const logs = store.get()

// if (logs.status !== 'completed') return logs;

const messages = logs.value;
		
	})
	

	import '@awesome.me/webawesome/dist/components/button/button.js';
	import '@awesome.me/webawesome/dist/components/select/select.js';
	import '@awesome.me/webawesome/dist/components/button-group/button-group.js';
	import '@awesome.me/webawesome/dist/components/switch/switch.js';
	import '@awesome.me/webawesome/dist/components/input/input.js';
	import { invoke } from '@tauri-apps/api/core';
	import { Signal } from 'signal-polyfill';

	let name = $state('');
	let greetMsg = $state('');

	async function greet(event: Event) {
		event.preventDefault();
		// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
		greetMsg = await invoke('greet', { name });
	}
</script>

<main class="container">
	{#if $clock.status === 'loading'}
	LOADING
	{/if}
	{#if $clock.status === 'completed'}
		{$clock.value}
	{/if}
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
