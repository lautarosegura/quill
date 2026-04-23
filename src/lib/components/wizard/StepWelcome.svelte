<script lang="ts">
	import IconFeather from '$lib/components/ui/icons/IconFeather.svelte';
	import IconMic from '$lib/components/ui/icons/IconMic.svelte';
</script>

<div class="flex h-full flex-col items-center justify-center text-center">
	<div
		class="flex h-16 w-16 items-center justify-center rounded-2xl"
		style="background: var(--accent-soft); color: var(--accent);"
	>
		<IconFeather size={32} stroke={1.75} />
	</div>
	<h1 class="text-base-strong mt-5 text-[24px] font-semibold tracking-tight">
		Bienvenido a Quill
	</h1>
	<p class="text-base-dim mt-2 max-w-[420px] text-[14px] leading-relaxed">
		Dictado por voz en <span class="text-base-strong font-medium">cualquier app</span>. Apretás un
		hotkey, hablás, soltás — el texto aparece donde tengas el cursor.
	</p>
	<p class="text-base-mute mt-6 flex items-center gap-2 text-[11.5px]">
		<IconMic size={13} />
		Te guiamos en 5 pasos cortos
	</p>
</div>
