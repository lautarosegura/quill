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
	<rect x="3" y="3" width="18" height="18" rx="2" />
	<path d="M3 9h18M9 21V9" />
</Icon>
