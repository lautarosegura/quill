<script lang="ts">
	import IconAlert from './icons/IconAlert.svelte';

	interface Props {
		onFix?: () => void;
	}
	let { onFix }: Props = $props();
</script>

<div
	class="flex w-full items-center gap-3 rounded-md border px-3.5 py-2.5"
	style="border-color: oklch(0.65 0.18 25 / 0.35); background: oklch(0.65 0.18 25 / 0.08)"
>
	<span style="color: oklch(0.72 0.18 25)">
		<IconAlert size={15} />
	</span>
	<div class="flex-1 text-[12px]" style="color: oklch(0.85 0.08 25)">
		<span class="font-semibold">Some permissions were revoked.</span>
		<span class="opacity-80"> Quill won't work until this is fixed.</span>
	</div>
	<button
		class="h-7 rounded px-3 text-[11.5px] font-semibold text-white"
		style="background: oklch(0.6 0.18 25)"
		onclick={onFix}
	>
		Fix now
	</button>
</div>
