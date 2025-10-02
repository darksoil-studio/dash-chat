<script lang="ts">
</script>

<main class="container">
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
