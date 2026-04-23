<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import favicon from '$lib/assets/favicon.svg';
	import { initDisplayServer } from '$lib/stores/platform.svelte';

	let { children } = $props();

	// Resolve display server (Win/mac/X11/Wayland) once so components that
	// need Wayland-specific UX can read `platform.isWayland` reactively.
	onMount(() => {
		initDisplayServer().catch(() => {
			// Silent — the fallback inside initDisplayServer handles it.
		});
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

{@render children()}
