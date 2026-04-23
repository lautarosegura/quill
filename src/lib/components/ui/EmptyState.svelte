<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		title: string;
		/** Plain-text description. Ignored if `descriptionSnippet` is provided. */
		description?: string;
		/** Rich description — accepts inline KeyCap / emphasis markup. */
		descriptionSnippet?: Snippet;
		/** Tiny fine-print row shown under the description, e.g. "Offline · Private". */
		detailsSnippet?: Snippet;
		/** Icon slot rendered centered inside the accent-soft circle. */
		children?: Snippet;
	}
	let { title, description, descriptionSnippet, detailsSnippet, children }: Props = $props();
</script>

<div class="flex w-full flex-col items-center justify-center gap-4 py-10 text-center">
	<div class="relative flex h-20 w-20 items-center justify-center">
		<div class="absolute inset-0 rounded-full" style="background: var(--accent-soft)"></div>
		<div
			class="absolute inset-2 rounded-full border"
			style="border-color: color-mix(in oklch, var(--accent) 30%, transparent)"
		></div>
		<div class="accent-text relative">
			{@render children?.()}
		</div>
	</div>
	<div>
		<h3 class="text-base-strong text-[15px] font-semibold">{title}</h3>
		{#if descriptionSnippet}
			<div
				class="text-base-dim mt-1.5 flex max-w-[320px] flex-wrap items-center justify-center gap-x-1.5 gap-y-1 text-[12.5px]"
				style="text-wrap: pretty"
			>
				{@render descriptionSnippet()}
			</div>
		{:else if description}
			<p class="text-base-dim mt-1.5 max-w-[280px] text-[12.5px]" style="text-wrap: pretty">
				{description}
			</p>
		{/if}
	</div>
	{#if detailsSnippet}
		<div class="text-base-mute flex items-center gap-2 text-[11px]">
			{@render detailsSnippet()}
		</div>
	{/if}
</div>
