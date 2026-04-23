<script lang="ts">
	import {
		Button,
		Toast,
		Spinner,
		Pill,
		Waveform,
		DotSpinner,
		RevokedBanner,
		EmptyState,
		KeyCap,
		Tooltip,
		Modal,
		StatusDot,
		IconMic,
		IconCopy,
		// Phase 4 primitives
		Toggle,
		Slider,
		Segmented,
		RadioCard,
		Dropdown,
		PasswordInput,
		KeyCapture,
		Accordion,
		OverlayPositionPicker
	} from '$lib/components/ui';
	import type { Keybind, OverlayPosition } from '$lib/types';

	let showModal = $state(false);
	const states = ['idle', 'recording', 'transcribing', 'injecting', 'error'] as const;

	// Phase 4 primitive state for the demos
	let toggleValue = $state(true);
	let sliderValue = $state(60);
	let segmentedValue = $state<'es' | 'en'>('es');
	let radioValue = $state<'local' | 'groq'>('local');
	let dropdownValue = $state<'a' | 'b' | 'c'>('a');
	let passwordValue = $state('');
	let hotkeyValue = $state<Keybind>({ modifiers: ['ctrl', 'shift'], key: 'Space' });
	let overlayPos = $state<OverlayPosition>('bottom-center');
</script>

<div class="bg-app text-base-strong flex h-screen flex-col gap-10 overflow-auto p-8">
	<header class="border-hair border-b pb-4">
		<h1 class="text-xl font-semibold">Quill · Design system</h1>
		<p class="text-base-dim mt-1 text-sm">
			Visual regression gallery. Hit this page after every UI change.
		</p>
	</header>

	<!-- Buttons -->
	<section>
		<h2 class="text-base-mute mb-3 text-sm font-semibold tracking-wider uppercase">Buttons</h2>
		<div class="flex flex-wrap items-center gap-3">
			<Button variant="primary">Primary</Button>
			<Button variant="secondary">Secondary</Button>
			<Button variant="ghost">Ghost</Button>
			<Button variant="danger">Danger</Button>
			<Button disabled>Disabled</Button>
			<Button size="sm">Small</Button>
		</div>
	</section>

	<!-- Status dots -->
	<section>
		<h2 class="text-base-mute mb-3 text-sm font-semibold tracking-wider uppercase">
			Status dots
		</h2>
		<div class="flex items-center gap-6">
			{#each states as s}
				<div class="flex items-center gap-2">
					<StatusDot state={s} />
					<span class="text-sm">{s}</span>
				</div>
			{/each}
		</div>
	</section>

	<!-- Overlay pill -->
	<section>
		<h2 class="text-base-mute mb-3 text-sm font-semibold tracking-wider uppercase">
			Overlay pill — 3 states
		</h2>
		<div class="flex flex-wrap items-center gap-6">
			<Pill state="recording" />
			<Pill state="transcribing" />
			<Pill state="error" message="No internet" onClose={() => {}} />
		</div>
	</section>

	<!-- Toasts -->
	<section>
		<h2 class="text-base-mute mb-3 text-sm font-semibold tracking-wider uppercase">Toasts</h2>
		<div class="flex w-[320px] flex-col gap-2">
			<Toast kind="success" message="Model downloaded successfully" />
			<Toast kind="info" message="Clipboard restored" />
			<Toast
				kind="error"
				message="Groq timed out, audio saved."
				action="Retry"
				onAction={() => {}}
			/>
		</div>
	</section>

	<!-- Empty state + banner -->
	<section class="grid grid-cols-2 gap-6">
		<div>
			<h2 class="text-base-mute mb-3 text-sm font-semibold tracking-wider uppercase">
				Empty state
			</h2>
			<div class="border-hair bg-panel rounded-lg border p-5">
				<EmptyState
					title="No dictations yet"
					description="Press Ctrl Shift Space anywhere to start your first dictation."
				>
					<IconMic size={32} stroke={1.5} />
				</EmptyState>
			</div>
		</div>
		<div>
			<h2 class="text-base-mute mb-3 text-sm font-semibold tracking-wider uppercase">
				Revoked banner
			</h2>
			<RevokedBanner onFix={() => {}} />
		</div>
	</section>

	<!-- Spinner + KeyCaps + Tooltip -->
	<section class="grid grid-cols-3 gap-6">
		<div>
			<h2 class="text-base-mute mb-3 text-sm font-semibold tracking-wider uppercase">
				Spinner
			</h2>
			<Spinner />
		</div>
		<div>
			<h2 class="text-base-mute mb-3 text-sm font-semibold tracking-wider uppercase">
				Key caps
			</h2>
			<div class="flex items-center gap-1.5">
				<KeyCap size="big">Ctrl</KeyCap>
				<span class="text-base-mute text-xs">+</span>
				<KeyCap size="big">Shift</KeyCap>
				<span class="text-base-mute text-xs">+</span>
				<KeyCap size="big">Space</KeyCap>
			</div>
			<div class="mt-3 flex items-center gap-1 opacity-75">
				<KeyCap>⌘</KeyCap>
				<KeyCap>K</KeyCap>
			</div>
		</div>
		<div>
			<h2 class="text-base-mute mb-3 text-sm font-semibold tracking-wider uppercase">
				Tooltip (hover)
			</h2>
			<Tooltip text="Copy transcription" shortcut="⌘C">
				<button class="border-hair bg-elev text-base-dim rounded-md border p-2">
					<IconCopy size={14} />
				</button>
			</Tooltip>
		</div>
	</section>

	<!-- Waveform + DotSpinner -->
	<section class="grid grid-cols-2 gap-6">
		<div>
			<h2 class="text-base-mute mb-3 text-sm font-semibold tracking-wider uppercase">
				Waveform
			</h2>
			<div class="border-hair bg-panel flex items-center justify-center rounded-lg border p-6">
				<Waveform active color="var(--accent)" />
			</div>
		</div>
		<div>
			<h2 class="text-base-mute mb-3 text-sm font-semibold tracking-wider uppercase">
				DotSpinner
			</h2>
			<div class="border-hair bg-panel flex items-center justify-center rounded-lg border p-6">
				<DotSpinner color="var(--accent)" />
			</div>
		</div>
	</section>

	<!-- Modal -->
	<section>
		<h2 class="text-base-mute mb-3 text-sm font-semibold tracking-wider uppercase">Modal</h2>
		<Button onclick={() => (showModal = true)}>Open confirm modal</Button>
		{#if showModal}
			<Modal
				title="Delete all history?"
				description="This will remove all 247 transcriptions permanently. This action can't be undone."
				confirmLabel="Delete all"
				cancelLabel="Cancel"
				destructive
				onConfirm={() => (showModal = false)}
				onCancel={() => (showModal = false)}
			/>
		{/if}
	</section>

	<!-- ========== Phase 4 primitives ========== -->

	<div class="border-hair border-b pt-6 pb-3">
		<h2 class="text-base-strong text-lg font-semibold">Phase 4 · Form primitives</h2>
	</div>

	<!-- Toggle + Slider -->
	<section class="grid grid-cols-2 gap-6">
		<div>
			<h2 class="text-base-mute mb-3 text-sm font-semibold tracking-wider uppercase">Toggle</h2>
			<div class="flex flex-col gap-3">
				<Toggle value={toggleValue} label="Auto-save" onChange={(v) => (toggleValue = v)} />
				<Toggle value={false} disabled label="Disabled" onChange={() => {}} />
			</div>
		</div>
		<div>
			<h2 class="text-base-mute mb-3 text-sm font-semibold tracking-wider uppercase">Slider</h2>
			<Slider
				value={sliderValue}
				min={10}
				max={120}
				step={5}
				unit="s"
				label="Duración máxima"
				onChange={(v) => (sliderValue = v)}
			/>
		</div>
	</section>

	<!-- Segmented + Dropdown -->
	<section class="grid grid-cols-2 gap-6">
		<div>
			<h2 class="text-base-mute mb-3 text-sm font-semibold tracking-wider uppercase">
				Segmented
			</h2>
			<Segmented
				value={segmentedValue}
				options={[
					{ value: 'es', label: '🇪🇸 Español' },
					{ value: 'en', label: '🇺🇸 English' }
				]}
				onChange={(v) => (segmentedValue = v)}
			/>
		</div>
		<div>
			<h2 class="text-base-mute mb-3 text-sm font-semibold tracking-wider uppercase">
				Dropdown
			</h2>
			<Dropdown
				value={dropdownValue}
				options={[
					{ value: 'a', label: 'Opción A' },
					{ value: 'b', label: 'Opción B' },
					{ value: 'c', label: 'Opción C' }
				]}
				onChange={(v) => (dropdownValue = v)}
			/>
		</div>
	</section>

	<!-- RadioCard -->
	<section>
		<h2 class="text-base-mute mb-3 text-sm font-semibold tracking-wider uppercase">RadioCard</h2>
		<RadioCard
			value={radioValue}
			options={[
				{
					value: 'local',
					title: 'Local (whisper.cpp)',
					description: 'Corre en tu máquina. Privado.'
				},
				{
					value: 'groq',
					title: 'Groq Cloud',
					description: 'Más rápido. Necesita internet.',
					badge: 'Recomendado'
				}
			]}
			onChange={(v) => (radioValue = v)}
		/>
	</section>

	<!-- PasswordInput + KeyCapture -->
	<section class="grid grid-cols-2 gap-6">
		<div>
			<h2 class="text-base-mute mb-3 text-sm font-semibold tracking-wider uppercase">
				PasswordInput
			</h2>
			<PasswordInput
				value={passwordValue}
				onChange={(v) => (passwordValue = v)}
				placeholder="gsk_..."
			/>
		</div>
		<div>
			<h2 class="text-base-mute mb-3 text-sm font-semibold tracking-wider uppercase">
				KeyCapture
			</h2>
			<KeyCapture value={hotkeyValue} onChange={(v) => (hotkeyValue = v)} />
		</div>
	</section>

	<!-- OverlayPositionPicker -->
	<section>
		<h2 class="text-base-mute mb-3 text-sm font-semibold tracking-wider uppercase">
			Overlay position picker
		</h2>
		<OverlayPositionPicker value={overlayPos} onChange={(v) => (overlayPos = v)} />
	</section>

	<!-- Accordion -->
	<section>
		<h2 class="text-base-mute mb-3 text-sm font-semibold tracking-wider uppercase">Accordion</h2>
		<div class="border-hair bg-panel rounded-lg border px-4">
			<Accordion title="Sección colapsable" defaultOpen>
				{#snippet children()}
					<p class="text-base-dim text-[12px] leading-relaxed">
						Este es el contenido dentro de un Accordion. Usa el elemento nativo
						<code>{'<details>'}</code> para a11y.
					</p>
				{/snippet}
			</Accordion>
			<Accordion title="Otra sección (cerrada por defecto)">
				{#snippet children()}
					<p class="text-base-dim text-[12px]">Este empieza cerrado. Click en el título.</p>
				{/snippet}
			</Accordion>
		</div>
	</section>
</div>
