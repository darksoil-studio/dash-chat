<script lang="ts">
	import '@awesome.me/webawesome/dist/components/icon/icon.js';
	import '@awesome.me/webawesome/dist/components/button/button.js';
</script>

<div class="column">
	<div class="top-bar">
		<wa-button
			class="circle"
			appearance="plain"
			onclick={() => {
				window.history.back();
			}}
		>
			<wa-icon name="arrow-left"> </wa-icon>
		</wa-button>
		<div style="flex: 1"></div>
		<wa-button class="circle" href="/add-friend" appearance="plain">
			<wa-icon name="user-plus"> </wa-icon>
		</wa-button>
	</div>
</div>
