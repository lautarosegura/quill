<script lang="ts">
	import type { Component, Snippet } from 'svelte';
	import IconChevron from './icons/IconChevron.svelte';

	interface Props {
		title: string;
		/** One-line summary of the current state (e.g. "Groq · Whisper Large v3"). */
		summary?: string;
		/** Lucide-style icon component rendered in the accent-tinted left box. */
		icon?: Component<{ size?: number; stroke?: number }>;
		defaultOpen?: boolean;
		children?: Snippet;
	}
	let { title, summary, icon: Ico, defaultOpen = false, children }: Props = $props();

	// svelte-ignore state_referenced_locally
	let open = $state(defaultOpen);
</script>

<div class="border-hair bg-panel overflow-hidden rounded-lg border">
	<button
		type="button"
		onclick={() => (open = !open)}
		class="bg-hover flex w-full items-center gap-3 px-4 py-3 text-left transition-colors"
	>
		{#if Ico}
			<div
				class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md"
				style="background: var(--accent-soft); color: var(--accent)"
			>
				<Ico size={14} stroke={1.75} />
			</div>
		{/if}
		<div class="min-w-0 flex-1">
			<div class="text-base-strong text-[13px] leading-tight font-semibold">{title}</div>
			{#if summary}
				<div class="text-base-mute mt-0.5 truncate text-[11.5px] leading-tight">{summary}</div>
			{/if}
		</div>
		<span class="text-base-mute shrink-0 transition-transform duration-150" style:transform={open ? 'rotate(90deg)' : 'rotate(0deg)'}>
			<IconChevron size={14} />
		</span>
	</button>
	{#if open}
		<div class="border-hair fade-in border-t px-4 pt-1 pb-4">
			{@render children?.()}
		</div>
	{/if}
</div>
