<script lang="ts">
	import '@awesome.me/webawesome/dist/components/avatar/avatar.js';
	import '@awesome.me/webawesome/dist/components/tooltip/tooltip.js';
	import {resizeAndExport} from '../utils/image'

	let avatarSize = 300;

	let { avatar = $bindable() }: {avatar: string | undefined} = $props();
	let uploading = $state(false);
	let avatarFilePicker: HTMLInputElement;

	function onAvatarUploaded() {
		uploading = true;
		if (avatarFilePicker.files && avatarFilePicker.files[0]) {
			const reader = new FileReader();
			reader.onload = e => {
				const img = new Image();
				img.crossOrigin = 'anonymous';
				img.onload = () => {
					avatar = resizeAndExport(img, avatarSize, avatarSize);
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

{#if avatar}
	<div
		class="column"
		style="align-items: center; height: 50px"
		onclick={() => {
			avatar = undefined;
		}}
	>
			<wa-avatar id="avatar" image={avatar} alt="Avatar" shape="circle" initials=""
			></wa-avatar>
		<wa-tooltip for="avatar">Clear</wa-tooltip
		>
	</div>
{:else}
	<div class="column" style="align-items: center;">
		<wa-button
			variant="default"
			disabled={uploading}
			loading={uploading}
			onclick={() => avatarFilePicker.click()}
		>
			<wa-icon name="plus" label="Add avatar image"></wa-icon>
		</wa-button>
	</div>
{/if}
