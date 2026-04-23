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
	<path d="M4 12h12" />
	<path d="m12 6 6 6-6 6" />
	<path d="M20 4v16" />
</Icon>
