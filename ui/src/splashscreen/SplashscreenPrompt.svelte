<script lang="ts">
	import Splashscreen from './Splashscreen.svelte';
	import { splashscreenDismissed } from './utils';

</script>

{#if $splashscreenDismissed}
	<slot></slot>
{:else}
	<Splashscreen></Splashscreen>
{/if}
