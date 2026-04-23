<script lang="ts">
	interface Props {
		value: number;
		min: number;
		max: number;
		step?: number;
		onChange: (next: number) => void;
		unit?: string;
		label?: string;
	}
	let { value, min, max, step = 1, onChange, unit, label }: Props = $props();

	const pct = $derived(((value - min) / (max - min)) * 100);

	function handleInput(e: Event & { currentTarget: HTMLInputElement }) {
		onChange(Number(e.currentTarget.value));
	}
</script>

<div class="flex flex-col gap-1.5">
	{#if label}
		<span class="text-base-dim text-[12px]">{label}</span>
	{/if}
	<div class="flex items-center gap-3">
		<input
			type="range"
			{min}
			{max}
			{step}
			{value}
			oninput={handleInput}
			class="quill-slider flex-1"
			style="--pct: {pct}%"
		/>
		<span
			class="text-base-dim shrink-0 text-right font-mono text-[11px]"
			style="min-width: 40px"
		>
			{value}{unit ?? ''}
		</span>
	</div>
</div>

<style>
	.quill-slider {
		-webkit-appearance: none;
		appearance: none;
		background: transparent;
		height: 14px;
		cursor: pointer;
		padding: 0;
		margin: 0;
	}
	.quill-slider:focus {
		outline: none;
	}

	/* WebKit track */
	.quill-slider::-webkit-slider-runnable-track {
		height: 3px;
		border-radius: 2px;
		background: linear-gradient(
			to right,
			var(--accent) 0%,
			var(--accent) var(--pct),
			var(--border-strong) var(--pct),
			var(--border-strong) 100%
		);
	}
	/* Firefox track */
	.quill-slider::-moz-range-track {
		height: 3px;
		border-radius: 2px;
		background: var(--border-strong);
	}
	.quill-slider::-moz-range-progress {
		height: 3px;
		border-radius: 2px;
		background: var(--accent);
	}

	/* WebKit thumb */
	.quill-slider::-webkit-slider-thumb {
		-webkit-appearance: none;
		appearance: none;
		width: 14px;
		height: 14px;
		border-radius: 50%;
		background: var(--accent);
		border: 2px solid var(--bg);
		margin-top: -5.5px;
		box-shadow: 0 1px 3px oklch(0 0 0 / 0.25);
		transition: transform 120ms ease;
	}
	.quill-slider::-webkit-slider-thumb:hover {
		transform: scale(1.1);
	}

	/* Firefox thumb */
	.quill-slider::-moz-range-thumb {
		width: 14px;
		height: 14px;
		border-radius: 50%;
		background: var(--accent);
		border: 2px solid var(--bg);
		box-shadow: 0 1px 3px oklch(0 0 0 / 0.25);
		transition: transform 120ms ease;
	}
	.quill-slider::-moz-range-thumb:hover {
		transform: scale(1.1);
	}
</style>
