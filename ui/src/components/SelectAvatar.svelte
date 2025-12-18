<script lang="ts">
	import '@awesome.me/webawesome/dist/components/avatar/avatar.js';
	import { resizeAndExport } from '../utils/image';
	import { wrapPathInSvg } from '@darksoil-studio/holochain-elements';
	import { mdiPlus } from '@mdi/js';
	import { m } from '$lib/paraglide/messages.js';
	import { Button, Fab } from 'konsta/svelte';

	let avatarSize = 300;

	let {
		value = $bindable(),
		defaultValue,
	}: { value?: string | undefined; defaultValue?: string | undefined } =
		$props();
	let uploading = $state(false);
	let avatarFilePicker: HTMLInputElement;

	if (!value) {
		value = defaultValue;
	}

	function onAvatarUploaded() {
		uploading = true;
		if (avatarFilePicker.files && avatarFilePicker.files[0]) {
			const reader = new FileReader();
			reader.onload = e => {
				const img = new Image();
				img.crossOrigin = 'anonymous';
				img.onload = () => {
					value = resizeAndExport(img, avatarSize, avatarSize);
					avatarFilePicker.value = '';

					uploading = false;
				};
				img.src = e.target?.result as string;
			};
			reader.readAsDataURL(avatarFilePicker.files[0]);
		}
	}
</script>

<input
	type="file"
	bind:this={avatarFilePicker}
	style="display: none"
	onchange={onAvatarUploaded}
/>

{#if value}
	<div
		class="column"
		style="align-items: center; height: 50px"
		onclick={() => avatarFilePicker.click()}
	>
		<wa-avatar id="avatar" image={value} alt="Avatar" shape="circle" initials=""
		></wa-avatar>
	</div>
{:else if defaultValue}
	<div
		class="column"
		style="align-items: center; height: 50px"
		onclick={() => avatarFilePicker.click()}
	>
		<wa-avatar
			id="avatar"
			image={defaultValue}
			alt="Avatar"
			shape="circle"
			initials=""
		></wa-avatar>
	</div>
{:else}
	<div class="column" style="align-items: center; height: 50px">
		<Button
			onclick={() => avatarFilePicker.click()}
			disabled={uploading}
			rounded
			style="border-radius: 50%; height: 46px; width: 46px"
		>
			<wa-icon src={wrapPathInSvg(mdiPlus)} label={m.addAvatarImage()}></wa-icon>
				
		</Button>
	</div>
{/if}
