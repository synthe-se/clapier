<script lang="ts">
	// The last requests the burrow answered, newest first: the rabbits' own (their bytecode, their
	// sounds) marked, the rest of the LAN on demand. A list of short facts, not a wide table: it
	// reads the same on a phone.
	import { m } from '#lib/paraglide/messages.js';
	import type { Hit } from '#lib/api.ts';
	import { ago, bytes, num } from '#lib/i18n.svelte.ts';
	import Icon from './Icon.svelte';

	interface Props {
		hits: Hit[];
	}
	let { hits }: Props = $props();

	const PAGE = 25;
	let onlyRabbits = $state(true);
	let all = $state(false);
	const kept = $derived(onlyRabbits ? hits.filter((h) => h.rabbit) : hits);
	const shown = $derived(all ? kept : kept.slice(0, PAGE));
</script>

<div class="group-head">
	<h2 class="group-title" id="journal-title">{m.journal_title()}</h2>
	<div class="segmented actions" role="group" aria-label={m.journal_filter()}>
		<button class="btn" aria-pressed={onlyRabbits} onclick={() => (onlyRabbits = true)}>{m.journal_rabbits()}</button>
		<button class="btn" aria-pressed={!onlyRabbits} onclick={() => (onlyRabbits = false)}>{m.journal_all()}</button>
	</div>
</div>

{#if shown.length}
	<ul class="list" aria-labelledby="journal-title">
		{#each shown as h, i (`${h.ago_ms}-${h.uri}-${i}`)}
			<li class="list-row hit" class:warn={h.status >= 400}>
				<span class="lead" class:rabbit={h.rabbit}><Icon name={h.rabbit ? 'rabbit' : 'globe'} size={16} label={h.rabbit ? m.journal_from_rabbit() : undefined} /></span>
				<span class="title mono"><span class="method">{h.method}</span> {h.uri}</span>
				<span class="meta">{h.status} · {bytes(h.bytes)}</span>
				<span class="second">{ago(h.ago_ms / 1000)} · <span class="mono">{h.peer}</span></span>
			</li>
		{/each}
	</ul>
	{#if kept.length > PAGE}
		<button class="link-btn more" onclick={() => (all = !all)}>
			{all ? m.journal_less() : m.journal_more({ count: num(kept.length) })}
		</button>
	{/if}
{:else}
	<div class="empty"><p>{onlyRabbits ? m.journal_empty_rabbits() : m.journal_empty()}</p></div>
{/if}

<style>
	.group-head .actions { margin-left: auto; }
	.segmented { gap: 0; }
	.segmented .btn { border-radius: 0; }
	.segmented .btn:first-child { border-radius: var(--radius) 0 0 var(--radius); }
	.segmented .btn:last-child { border-radius: 0 var(--radius) var(--radius) 0; margin-left: -1px; }
	.hit { cursor: default; }
	.hit .title { overflow-wrap: anywhere; }
	.method { color: var(--ink-muted); }
	.lead { color: var(--ink-muted); }
	.lead.rabbit { color: var(--accent); }
	.more { margin-top: var(--s-3); }
</style>
