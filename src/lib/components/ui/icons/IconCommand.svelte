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
	<path d="M18 3a3 3 0 0 0-3 3v12a3 3 0 0 0 3 3 3 3 0 0 0 3-3 3 3 0 0 0-3-3H6a3 3 0 0 0-3 3 3 3 0 0 0 3 3 3 3 0 0 0 3-3V6a3 3 0 0 0-3-3 3 3 0 0 0-3 3 3 3 0 0 0 3 3h12a3 3 0 0 0 3-3 3 3 0 0 0-3-3z" />
</Icon>
