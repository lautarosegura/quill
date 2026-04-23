<script lang="ts">
	import type { OverlayPosition } from '$lib/types';

	interface Props {
		value: OverlayPosition;
		onChange: (next: OverlayPosition) => void;
	}
	let { value, onChange }: Props = $props();

	const POSITIONS: OverlayPosition[] = [
		'top-left',
		'top-center',
		'top-right',
		'bottom-left',
		'bottom-center',
		'bottom-right'
	];

	const LABEL: Record<OverlayPosition, string> = {
		'top-left': 'Arriba izquierda',
		'top-center': 'Arriba centro',
		'top-right': 'Arriba derecha',
		'bottom-left': 'Abajo izquierda',
		'bottom-center': 'Abajo centro',
		'bottom-right': 'Abajo derecha'
	};

	function pillStyle(pos: OverlayPosition): string {
		const [vertical, horizontal] = pos.split('-') as [
			'top' | 'bottom',
			'left' | 'center' | 'right'
		];
		const props: string[] = ['position: absolute', 'width: 18px', 'height: 5px', 'border-radius: 3px'];
		if (vertical === 'top') props.push('top: 6px');
		else props.push('bottom: 6px');
		if (horizontal === 'left') props.push('left: 6px');
		else if (horizontal === 'right') props.push('right: 6px');
		else {
			props.push('left: 50%');
			props.push('transform: translateX(-50%)');
		}
		return props.join('; ');
	}
</script>

<div class="grid grid-cols-3 gap-2" role="radiogroup" aria-label="Posición del overlay">
	{#each POSITIONS as pos (pos)}
		{@const selected = pos === value}
		<button
			type="button"
			role="radio"
			aria-checked={selected}
			aria-label={LABEL[pos]}
			title={LABEL[pos]}
			onclick={() => onChange(pos)}
			class="bg-elev relative overflow-hidden rounded-md border transition-colors"
			style="aspect-ratio: 16 / 10; border-color: {selected
				? 'var(--accent)'
				: 'var(--border)'}; box-shadow: {selected ? '0 0 0 2px var(--accent-ring)' : 'none'}"
		>
			<!-- Mock screen content: 3 subtle lines suggest an app window.
			     Gives the tile a "this is a screen" feel so the pill's position
			     reads visually, not just conceptually. -->
			<div class="pointer-events-none absolute inset-2 flex flex-col gap-1">
				<div
					class="h-0.5 rounded-full"
					style="width: 30%; background: var(--border-strong)"
				></div>
				<div
					class="h-0.5 rounded-full"
					style="width: 60%; background: var(--border-strong)"
				></div>
				<div
					class="h-0.5 rounded-full"
					style="width: 45%; background: var(--border-strong)"
				></div>
			</div>
			<!-- Pill preview itself -->
			<span
				style="{pillStyle(pos)}; background: {selected ? 'var(--accent)' : 'var(--text-mute)'}"
			></span>
		</button>
	{/each}
</div>
