<script lang="ts">
	import Icon from '../Icon.svelte';
	interface Props {
		size?: number;
		stroke?: number;
		class?: string;
	}
	let { size, stroke, class: className }: Props = $props();
</script>

<Icon {size} {stroke} class={className}>
	<rect x="7" y="4" width="10" height="16" rx="2" />
	<path d="M9 4a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2" />
</Icon>
