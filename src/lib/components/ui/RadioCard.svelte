<script lang="ts" generics="T extends string">
	interface Props {
		value: T;
		options: Array<{
			value: T;
			title: string;
			description?: string;
			badge?: string;
		}>;
		onChange: (next: T) => void;
	}
	let { value, options, onChange }: Props = $props();
</script>

<div class="flex flex-col gap-2">
	{#each options as opt (opt.value)}
		{@const selected = opt.value === value}
		<button
			type="button"
			role="radio"
			aria-checked={selected}
			onclick={() => onChange(opt.value)}
			class="relative w-full rounded-lg border p-4 text-left transition-colors {selected
				? 'bg-panel'
				: 'border-hair bg-elev bg-hover'}"
			style={selected ? 'border-color: var(--accent)' : ''}
		>
			<div class="flex items-start justify-between gap-3">
				<div class="min-w-0 flex-1">
					<div class="flex items-center gap-2">
						<span class="text-base-strong text-[13px] font-semibold">{opt.title}</span>
						{#if opt.badge}
							<span
								class="accent-soft accent-text rounded px-2 py-0.5 text-[10.5px] font-semibold tracking-wider uppercase"
							>
								{opt.badge}
							</span>
						{/if}
					</div>
					{#if opt.description}
						<div class="text-base-dim mt-0.5 text-[12px]">{opt.description}</div>
					{/if}
				</div>
				{#if selected}
					<span
						class="mt-1 inline-block h-2.5 w-2.5 shrink-0 rounded-full"
						style="background: var(--accent)"
						aria-hidden="true"
					></span>
				{/if}
			</div>
		</button>
	{/each}
</div>
