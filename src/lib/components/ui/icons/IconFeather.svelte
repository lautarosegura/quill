<script lang="ts">
	// Quill brand logo — matches exactly the silhouette drawn by
	// src-tauri/tools/make_icons.ps1's `Draw-Quill` routine, so the in-app
	// icon (sidebar/titlebar/wizard brand) is the same glyph as the Start
	// menu / taskbar / tray icons. Canvas is 512x512 in design space, with
	// the same Bezier + line sequence as the PowerShell drawing.
	//
	// The `stroke` prop is accepted for API compatibility with the previous
	// Lucide-wrapped icon; it's unused here (the logo is a filled shape).
	interface Props {
		size?: number;
		stroke?: number;
		class?: string;
	}
	let { size = 16, class: className = '' }: Props = $props();
</script>

<svg
	width={size}
	height={size}
	viewBox="0 0 512 512"
	fill="currentColor"
	stroke="none"
	xmlns="http://www.w3.org/2000/svg"
	class={className}
	aria-hidden="true"
>
	<!--
		Points mirror the PS script variables:
		p1 (400,60) tip → c1 (510,200) / c2 (470,360) → p2 (200,430) blade base →
		L p3 (110,470) nib → L p4 (150,440) shaft start →
		c3 (260,260) / c4 (340,140) → back to p1.
	-->
	<path
		d="M 400 60 C 510 200 470 360 200 430 L 110 470 L 150 440 C 260 260 340 140 400 60 Z"
	/>
</svg>
