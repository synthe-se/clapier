<script lang="ts">
	// The burrow, on one screen: the door (who serves, since when), the rabbits and their
	// gestures, then the journal. Asked every few seconds while looked at (live.svelte.ts).
	import { m } from '#lib/paraglide/messages.js';
	import '#lib/i18n.svelte.ts';
	import { live } from '#lib/live.svelte.ts';
	import { ui } from '#lib/ui.svelte.ts';
	import { ago, duration, num } from '#lib/i18n.svelte.ts';
	import { AWAKE_S } from '#lib/format.ts';
	import Header from '#lib/components/Header.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Journal from '#lib/components/Journal.svelte';
	import RabbitTile from '#lib/components/RabbitTile.svelte';
	import Toasts from '#lib/components/Toasts.svelte';

	$effect(() => live.start());

	const s = $derived(live.status);
	// a rabbit only heard pulsing before it named itself still counts once: by IP
	// and a rabbit is listed once, whatever the burrow says: a repeated key would take the page down
	const rabbits = $derived(s ? [...new Map(s.rabbits.map((r) => [r.mac ?? r.ip, r])).values()].sort((a, b) => Number(!a.mac) - Number(!b.mac) || a.ip.localeCompare(b.ip, undefined, { numeric: true })) : []);
	const awake = $derived(rabbits.filter((r) => r.pulse_s !== null && r.pulse_s <= AWAKE_S).length);
</script>

<nav class="skip" aria-label={m.skip_to_content()}><a href="#main">{m.skip_to_content()}</a></nav>
<Header />
<main id="main" tabindex="-1">
	<section class="door" aria-labelledby="door-title">
		<div class="page door-inner">
			<h1 id="door-title" class="sr-only">{m.brand_name()}</h1>
			{#if s}
				<ul class="door-facts">
					<li><Icon name="rabbit" size={16} />{m.door_awake({ awake: num(awake), count: num(rabbits.length) })}</li>
					<li><Icon name="clock" size={16} />{m.door_uptime({ uptime: duration(s.uptime_s) })}</li>
					<li><span class="chip" title={s.serving}>clapier {s.version}</span></li>
				</ul>
			{/if}
		</div>
	</section>

	<div class="page body">
		{#if live.lost}
			<div class="banner warn" role="status">
				<Icon name="wifi-off" />
				<p>{s ? m.live_lost_body() : m.live_never()}</p>
				<button class="btn" onclick={() => live.refresh()}><Icon name="refresh-cw" />{m.retry()}</button>
			</div>
		{/if}

		{#if s}
			<div class="cols">
			<section aria-labelledby="rabbits-title" class="section">
				<div class="group-head">
					<h2 class="group-title" id="rabbits-title">{m.rabbits_title()}</h2>
				</div>
				{#if rabbits.length}
					<div class="tiles">
						{#each rabbits as r (r.mac ?? r.ip)}
							<RabbitTile rabbit={r} chors={s.chors} firmwares={s.firmwares} />
						{/each}
					</div>
				{:else}
					<div class="empty">
						<p class="empty-title">{m.rabbits_empty()}</p>
						<p>{m.rabbits_empty_hint()}</p>
					</div>
				{/if}
			</section>

			<section aria-labelledby="journal-title" class="section journal">
				<Journal hits={s.journal} />
			</section>
			</div>
		{:else if !live.lost}
			<p class="muted loading">{m.loading()}</p>
		{/if}
	</div>
</main>
<Toasts />
<div class="sr-only" role="status" aria-live="polite" aria-atomic="true">{ui.polite}</div>
<div class="sr-only" role="alert">{ui.assertive}</div>

<style>
	main { padding-bottom: var(--s-7); }
	main:focus { outline: none; }

	/* the door: a slim band on the brand's violet, in both themes */
	.door { background: var(--brand); color: var(--on-brand); }
	.door-inner { display: flex; flex-wrap: wrap; align-items: baseline; gap: var(--s-2) var(--s-5); padding-block: var(--s-3); }
	.door-facts { list-style: none; margin: 0; padding: 0; display: flex; flex-wrap: wrap; gap: var(--s-1) var(--s-4); font: var(--t-secondary); }
	.door-facts li { display: inline-flex; align-items: center; gap: 6px; min-width: 0; }
	.door-facts .chip { background: rgb(255 255 255 / 16%); color: inherit; }

	.body { display: grid; gap: var(--s-6); padding-top: var(--s-5); }
	.section { display: grid; gap: var(--s-3); min-width: 0; }
	/* the rabbits, then the journal: stacked on a phone, side by side once there is room, the
	   journal a quieter column that follows the page */
	.cols { display: grid; gap: var(--s-6); align-items: start; }
	@media (min-width: 1100px) {
		.cols { grid-template-columns: minmax(0, 1fr) minmax(20rem, 28rem); gap: var(--s-6); }
		.journal { position: sticky; top: calc(56px + var(--s-4)); max-height: calc(100dvh - 56px - var(--s-5)); overflow: auto; }
	}
	.empty-title { font: var(--t-group); color: var(--ink); margin-bottom: var(--s-2); }
	.loading { margin: 0; }
</style>
