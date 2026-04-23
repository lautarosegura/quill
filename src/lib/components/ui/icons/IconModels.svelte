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
	<rect x="3" y="3" width="7" height="7" rx="1.5" />
	<rect x="14" y="3" width="7" height="7" rx="1.5" />
	<rect x="3" y="14" width="7" height="7" rx="1.5" />
	<rect x="14" y="14" width="7" height="7" rx="1.5" />
</Icon>
