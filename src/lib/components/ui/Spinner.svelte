<script lang="ts">
	interface Props {
		size?: number;
	}
	let { size = 22 }: Props = $props();
</script>

<svg width={size} height={size} viewBox="0 0 24 24" fill="none" class="accent-text animate-spin">
	<circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="2.5" opacity="0.2" />
	<path d="M21 12a9 9 0 0 0-9-9" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" />
</svg>
