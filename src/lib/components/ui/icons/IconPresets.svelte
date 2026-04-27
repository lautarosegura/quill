<script lang="ts">
	import Icon from '../Icon.svelte';
	interface Props {
		size?: number;
		stroke?: number;
		class?: string;
	}
	let { size, stroke, class: className }: Props = $props();
</script>

<!-- Stack-of-cards glyph: front rectangle + an outlined "back" card peeking
     up-left, suggesting multiple saved configurations the user can switch
     between. From the v0.3.0 Claude Design bundle (option A, recommended). -->
<Icon {size} {stroke} class={className}>
	<rect x="7" y="7" width="13" height="13" rx="2" />
	<path d="M16 4H6a2 2 0 0 0-2 2v10" />
</Icon>
