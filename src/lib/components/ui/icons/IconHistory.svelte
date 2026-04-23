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
	<path d="M3 12a9 9 0 1 0 3-6.7" />
	<path d="M3 4v5h5" />
	<path d="M12 7v5l3 2" />
</Icon>
