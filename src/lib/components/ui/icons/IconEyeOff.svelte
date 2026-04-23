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
	<path d="M3 3l18 18" />
	<path d="M10.6 6.2A10.4 10.4 0 0 1 12 6c6.5 0 10 6 10 6a17.7 17.7 0 0 1-3.2 4.1" />
	<path d="M6.6 6.6A17.7 17.7 0 0 0 2 12s3.5 6 10 6a10.4 10.4 0 0 0 4.3-.9" />
	<path d="M9.9 9.9a3 3 0 0 0 4.2 4.2" />
</Icon>
