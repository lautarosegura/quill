<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	onMount(() => {
		goto('/main');
	});
</script>

<div class="p-8 text-sm text-neutral-500">Loading Quill…</div>
