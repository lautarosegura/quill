<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { Pill } from '$lib/components/ui';
	import { status, subscribeToStatus, unsubscribeFromStatus } from '$lib/stores/status.svelte';

	onMount(() => {
		subscribeToStatus();
	});

	onDestroy(() => {
		unsubscribeFromStatus();
	});

	const visibleState = $derived.by(() => {
		const s = status.value;
		if (s.state === 'recording') return 'recording' as const;
		if (s.state === 'transcribing' || s.state === 'injecting') return 'transcribing' as const;
		if (s.state === 'error') return 'error' as const;
		if (s.state === 'cancelled') return 'cancelled' as const;
		return null;
	});

	const errorMessage = $derived(
		status.value.state === 'error' ? status.value.message : undefined
	);
</script>

<div class="flex h-screen w-screen items-center justify-center">
	{#if visibleState}
		<Pill state={visibleState} message={errorMessage} />
	{/if}
</div>

<style>
	/* The overlay window is Tauri-transparent (tauri.conf.json → transparent:
	   true). But `body.bg-app` from app.html and the `html, body { background:
	   var(--bg) }` rule in app.css both paint a filled background. Override
	   them all explicitly — we also target `body.bg-app` by class so we beat
	   its specificity (a tag `!important` would otherwise tie against the
	   class selector in some ordering edge cases). */
	:global(html),
	:global(body),
	:global(body.bg-app) {
		background: transparent !important;
		background-color: transparent !important;
	}
</style>
