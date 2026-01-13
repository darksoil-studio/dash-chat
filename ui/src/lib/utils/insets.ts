
export async function setupInsets() {
	const { getBottomInset, getTopInset, onKeyboardHidden, onKeyboardShown } =
		await import('@saurl/tauri-plugin-safe-area-insets-css-api');
	onKeyboardShown(async () => {
		const bi = await getBottomInset();
		const top = await getTopInset();
		document.documentElement.style = `--safe-area-inset-bottom: ${bi?.inset}px; --safe-area-inset-top: ${top?.inset}px`;
	});
}
