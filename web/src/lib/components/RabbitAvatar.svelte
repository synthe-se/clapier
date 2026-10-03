<script lang="ts">
	// The rabbit as the brand draws it, alive: its ears where its /status says they are (0..16
	// around the turn), its belly LEDs lit while it pulses, dimmed while it is silent. Decoration
	// only: the tile says all of it in words.
	import { earAngle } from '#lib/format.ts';

	interface Props {
		ears: [number, number] | null;
		awake: boolean;
		size?: number;
	}
	let { ears, awake, size = 56 }: Props = $props();
	const left = $derived(ears ? earAngle(ears[0], true) : -10);
	const right = $derived(ears ? earAngle(ears[1], false) : 38);
</script>

<svg class="avatar" class:awake width={size} height={size} viewBox="0 0 48 48" aria-hidden="true" focusable="false">
	<g class="ear" transform="rotate({left} 21 19.5)">
		<ellipse cx="21" cy="11.3" rx="3.3" ry="8.2" class="fur" />
		<ellipse cx="21" cy="12.1" rx="1.4" ry="5.4" class="inner" />
	</g>
	<g class="ear" transform="rotate({right} 27 19.5)">
		<ellipse cx="27" cy="11.3" rx="3.3" ry="8.2" class="fur" />
	</g>
	<path d="M13 44C13 27 17 19.5 24 19.5S35 27 35 44Z" class="fur" />
	<path d="M18.2 29a2.2 2.2 0 0 1 4.2 0M25.6 29a2.2 2.2 0 0 1 4.2 0" class="eyes" />
	<ellipse cx="16.9" cy="32.2" rx="1.9" ry="1.1" class="blush" />
	<ellipse cx="31.1" cy="32.2" rx="1.9" ry="1.1" class="blush" />
	<circle cx="19.5" cy="38" r="1.5" class="led a" />
	<circle cx="24" cy="38.6" r="1.5" class="led b" />
	<circle cx="28.5" cy="38" r="1.5" class="led c" />
</svg>

<style>
	.avatar { flex: none; display: block; }
	.ear { transition: transform 0.6s ease-in-out; }
	.fur { fill: #f2f0ea; stroke: var(--accent-soft); stroke-width: 0.8; }
	.inner { fill: #b3a4ff; }
	.eyes { fill: none; stroke: #4f37b0; stroke-width: 1.7; stroke-linecap: round; }
	.blush { fill: #e6a49a; opacity: 0.85; }
	.led { fill: #c9c5bb; }
	.awake .led.a { fill: #f0b43c; }
	.awake .led.b { fill: #5fc08a; }
	.awake .led.c { fill: #f08a7e; }
	.awake .led { animation: breathe 3s ease-in-out infinite; }
	.awake .led.b { animation-delay: 0.4s; }
	.awake .led.c { animation-delay: 0.8s; }
	@keyframes breathe { 50% { opacity: 0.45; } }
</style>
