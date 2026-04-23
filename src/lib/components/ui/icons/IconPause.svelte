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
	<rect x="6" y="5" width="4" height="14" rx="1" />
	<rect x="14" y="5" width="4" height="14" rx="1" />
</Icon>
