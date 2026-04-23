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
	<path d="M4 19.5V6a2 2 0 0 1 2-2h14v16H6.5a2.5 2.5 0 0 0 0 5H20" />
	<path d="M8 8h8M8 12h6" />
</Icon>
