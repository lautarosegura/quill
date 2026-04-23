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
	<path d="M3 3v18h18" />
	<path d="M7 15l4-4 3 3 5-6" />
</Icon>
