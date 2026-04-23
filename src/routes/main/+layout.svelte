<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { invoke } from '@tauri-apps/api/core';
	import { Sidebar, TitleBar } from '$lib/components/ui';
	import type { NavId } from '$lib/components/ui/Sidebar.svelte';
	import { config } from '$lib/stores/config.svelte';
	import { theme } from '$lib/stores/theme.svelte';

	let { children } = $props();

	onMount(() => {
		config.load();
		theme.init();
	});

	const PATH_TO_NAV: Record<string, NavId> = {
		'/main': 'historial',
		'/main/settings': 'settings',
		'/main/vocabulario': 'vocabulario',
		'/main/modelos': 'modelos',
		'/main/uso': 'uso'
	};

	const NAV_TO_PATH: Record<NavId, string> = {
		historial: '/main',
		settings: '/main/settings',
		vocabulario: '/main/vocabulario',
		modelos: '/main/modelos',
		uso: '/main/uso'
	};

	const activeNav = $derived<NavId>(PATH_TO_NAV[page.url.pathname] ?? 'historial');

	function navigate(id: NavId) {
		goto(NAV_TO_PATH[id]);
	}

	function quickRecord() {
		invoke('trigger_test_dictation').catch(console.error);
	}
</script>

<div class="flex h-screen flex-col">
	<TitleBar />
	<div class="flex min-h-0 flex-1">
		<Sidebar
			{activeNav}
			onNavigate={navigate}
			onQuickRecord={quickRecord}
		/>
		<main class="flex-1 overflow-y-auto">
			{@render children()}
		</main>
	</div>
</div>
