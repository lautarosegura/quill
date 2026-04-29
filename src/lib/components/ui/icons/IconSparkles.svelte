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
	<path d="M12 3l1.6 4.4L18 9l-4.4 1.6L12 15l-1.6-4.4L6 9l4.4-1.6L12 3z" />
	<path d="M19 14l.7 1.8L21.5 16.5l-1.8.7L19 19l-.7-1.8L16.5 16.5l1.8-.7L19 14z" />
	<path d="M5 16l.5 1.3L6.8 17.8l-1.3.5L5 19.5l-.5-1.2L3.2 17.8l1.3-.5L5 16z" />
</Icon>
