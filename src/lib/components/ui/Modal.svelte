<script lang="ts">
	import Button from './Button.svelte';

	interface Props {
		title: string;
		description?: string;
		confirmLabel?: string;
		cancelLabel?: string;
		destructive?: boolean;
		onConfirm: () => void;
		onCancel: () => void;
	}
	let {
		title,
		description,
		confirmLabel = 'Confirmar',
		cancelLabel = 'Cancelar',
		destructive = false,
		onConfirm,
		onCancel
	}: Props = $props();
</script>

<div
	class="fixed inset-0 z-50 flex items-center justify-center"
	style="background: oklch(0 0 0 / 0.4)"
>
	<div
		class="border-hair-strong bg-panel w-[340px] rounded-lg border p-5"
		style="box-shadow: 0 30px 60px -20px oklch(0 0 0 / 0.5)"
	>
		<h3 class="text-base-strong text-[14px] font-semibold">{title}</h3>
		{#if description}
			<p class="text-base-dim mt-1.5 text-[12px]" style="text-wrap: pretty">{description}</p>
		{/if}
		<div class="mt-4 flex items-center justify-end gap-2">
			<Button variant="ghost" onclick={onCancel}>{cancelLabel}</Button>
			<Button variant={destructive ? 'danger' : 'primary'} onclick={onConfirm}>
				{confirmLabel}
			</Button>
		</div>
	</div>
</div>
