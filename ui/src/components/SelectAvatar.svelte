<script lang="ts">
	import '@awesome.me/webawesome/dist/components/avatar/avatar.js';
	import '@awesome.me/webawesome/dist/components/tooltip/tooltip.js';
	import { resizeAndExport } from '../utils/image';
	import { wrapPathInSvg } from '@darksoil-studio/holochain-elements';
	import { mdiPlus } from '@mdi/js';

	let avatarSize = 300;

	let { avatar = $bindable() }: { avatar: string | undefined } = $props();
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
		onclick={() => avatarFilePicker.click()}
	>
		<wa-avatar
			id="avatar"
			image={avatar}
			alt="Avatar"
			shape="circle"
			initials=""
		></wa-avatar>
	</div>
{:else}
	<div class="column" style="align-items: center;">
		<wa-button
			class="circle"
			variant="default"
			disabled={uploading}
			loading={uploading}
			onclick={() => avatarFilePicker.click()}
		>
			<wa-icon src={wrapPathInSvg(mdiPlus)} label="Add avatar image"></wa-icon>
		</wa-button>
	</div>
{/if}
