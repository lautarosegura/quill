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
	<rect x="9" y="3" width="6" height="12" rx="3" />
	<path d="M5 11a7 7 0 0 0 14 0" />
	<path d="M12 18v3" />
</Icon>
