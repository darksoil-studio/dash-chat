<script lang="ts">
	import { wrapPathInSvg } from '@darksoil-studio/holochain-elements';
	import { mdiAccountMultiplePlus, mdiAccountPlus, mdiArrowLeft } from '@mdi/js';
</script>

<div class="top-bar">
	<wa-button class="circle" href="/" appearance="plain">
		<wa-icon src={wrapPathInSvg(mdiArrowLeft)}> </wa-icon>
	</wa-button>
	<span class="title">New message </span>
</div>

<div class="column center-in-desktop" style="gap: var(--wa-space-l)">
	<div class="column" style="gap: var(--wa-space-m)">
		<wa-button href="/add-contact">
			<wa-icon slot="start" src={wrapPathInSvg(mdiAccountPlus)}> </wa-icon>
			Add contact
		</wa-button>

		<wa-button href="/new-group">
			<wa-icon slot="start" src={wrapPathInSvg(mdiAccountMultiplePlus)}> </wa-icon>
			New group
		</wa-button>
	</div>

	<div class="column">
		<span class="title">Contacts</span>
	</div>
</div>
