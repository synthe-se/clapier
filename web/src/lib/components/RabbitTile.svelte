<script lang="ts">
	// One rabbit: who it is, its state in words, what the wire said about it, then its gestures.
	// Gestures only for a rabbit the burrow can name (a MAC): the server aims the radio at
	// nothing else. Every outcome is said once, in a toast and its live region.
	import { Accordion, Slider } from 'bits-ui';
	import { m } from '#lib/paraglide/messages.js';
	import { api, type Rabbit } from '#lib/api.ts';
	import { ui } from '#lib/ui.svelte.ts';
	import { ago, bytes, duration } from '#lib/i18n.svelte.ts';
	import { AWAKE_S, earPose, ears as parseEars, mac as formatMac } from '#lib/format.ts';
	import { live } from '#lib/live.svelte.ts';
	import ConfirmDialog from './ConfirmDialog.svelte';
	import Icon, { type IconName } from './Icon.svelte';
	import RabbitAvatar from './RabbitAvatar.svelte';
	import Select from './Select.svelte';

	interface Props {
		rabbit: Rabbit;
		chors: string[];
		say: boolean;
	}
	let { rabbit, chors, say }: Props = $props();

	const name = $derived(rabbit.name ?? m.rabbit_unnamed());
	const awake = $derived(rabbit.pulse_s !== null && rabbit.pulse_s <= AWAKE_S);
	const position = $derived(parseEars(rabbit.ears));
	const id = $props.id();

	// ── what the wire said, in words ──
	const condition = $derived.by(() => {
		if (awake) return { text: m.rabbit_awake(), warn: false };
		if (rabbit.pulse_s !== null) return { text: m.rabbit_silent({ ago: ago(rabbit.pulse_s) }), warn: true };
		if (rabbit.boot_s !== null) return { text: m.rabbit_booted({ ago: ago(rabbit.boot_s) }), warn: false };
		return { text: m.rabbit_heard(), warn: false };
	});
	const POSE = { up: m.ear_up, tilted: m.ear_tilted, flat: m.ear_flat, down: m.ear_down } as const;
	const earWords = (p: number) => POSE[earPose(p)]();
	const audio = (v: string | null) => (v === null ? null : v === 'idle' ? m.rabbit_idle() : v);
	const facts = $derived(
		[
			{ icon: 'activity', label: m.fact_uptime(), value: rabbit.uptime_s === null ? null : duration(rabbit.uptime_s) },
			{ icon: 'wifi', label: m.fact_signal(), value: rabbit.rssi === null ? null : m.fact_rssi({ rssi: rabbit.rssi }) },
			{ icon: 'ear', label: m.fact_ears(), value: position ? m.fact_ears_value({ left: earWords(position[0]), right: earWords(position[1]) }) : null },
			{ icon: 'music', label: m.fact_audio(), value: audio(rabbit.audio) },
			{ icon: 'sparkles', label: m.fact_choreo(), value: audio(rabbit.choreo) },
			{ icon: 'refresh-cw', label: m.fact_boot(), value: rabbit.boot_s === null ? null : ago(rabbit.boot_s) }
		] satisfies { icon: IconName; label: string; value: string | null }[]
	);

	// ── gestures ──
	let busy = $state<string | null>(null);

	async function gesture(key: string, run: () => Promise<string>) {
		busy = key;
		try {
			ui.toast(await run());
		} catch (e) {
			ui.fail(e);
		} finally {
			busy = null;
		}
	}

	const said = (reply: string | null) => (reply === null ? m.reply_silent({ name }) : m.reply_said({ name, reply }));
	const ctl = (key: string, cmd: string) => gesture(key, async () => said((await api.ctl(rabbit.ip, cmd)).reply));

	const quick: { key: string; cmd: string; icon: IconName; label: () => string }[] = [
		{ key: 'ping', cmd: 'ping', icon: 'radio', label: m.action_ping },
		{ key: 'dance', cmd: 'dance', icon: 'sparkles', label: m.action_salute },
		{ key: 'stop', cmd: 'stop', icon: 'volume-x', label: m.action_stop }
	];

	let chor = $state('');
	const chorOptions = $derived(chors.map((c) => ({ value: c, label: c.split('/').pop()!.replace(/\.chor$/, '') })));

	let cdl = $state('');
	const compose = () =>
		gesture('compose', async () => {
			const r = await api.dance(rabbit.mac!, rabbit.ip, cdl);
			return r.reply === null ? m.dance_installed_silent({ bytes: bytes(r.bytes ?? 0), name }) : m.dance_installed({ bytes: bytes(r.bytes ?? 0), name });
		});

	const colors: { hex: string; label: () => string }[] = [
		{ hex: '7c5cff', label: m.color_violet },
		{ hex: '2b7bff', label: m.color_blue },
		{ hex: '2fd36b', label: m.color_green },
		{ hex: 'ffd23f', label: m.color_yellow },
		{ hex: 'ff8a1f', label: m.color_orange },
		{ hex: 'ff4fa3', label: m.color_pink },
		{ hex: 'ffffff', label: m.color_white },
		{ hex: '000000', label: m.color_off }
	];

	let left = $state(0);
	let right = $state(0);
	let earsTouched = $state(false);
	$effect(() => {
		// follow the rabbit until the person takes the sliders
		if (position && !earsTouched) [left, right] = position;
	});

	// renamed in place: the title becomes a field, Enter keeps it, Escape leaves it
	let renaming = $state(false);
	let draft = $state('');
	function startRename() {
		draft = rabbit.name ?? '';
		renaming = true;
	}
	async function rename(e: SubmitEvent) {
		e.preventDefault();
		busy = 'name';
		try {
			await api.name(rabbit.mac!, draft);
			renaming = false;
			await live.refresh();
			ui.toast(draft.trim() ? m.name_saved({ name: draft.trim() }) : m.name_forgotten());
		} catch (err) {
			ui.fail(err);
		} finally {
			busy = null;
		}
	}
	const focus = (el: HTMLElement) => el.focus();

	let sentence = $state('');
	const speak = () =>
		gesture('say', async () => {
			await api.say(sentence);
			sentence = '';
			return m.say_started();
		});
</script>

<article class="tile" aria-labelledby="{id}-name">
	<div class="head">
		<div class="glyph" class:awake><RabbitAvatar ears={position} {awake} /></div>
		<div class="text">
			{#if renaming}
				<form class="rename" onsubmit={rename}>
					<label class="sr-only" for="{id}-rename">{m.name_label()}</label>
					<input id="{id}-rename" bind:value={draft} maxlength="32" placeholder={m.name_placeholder()} autocomplete="off" onkeydown={(e) => e.key === 'Escape' && (renaming = false)} {@attach focus} />
					<button class="btn primary" disabled={busy !== null}><Icon name="check" busy={busy === 'name'} />{m.name_save()}</button>
					<button type="button" class="btn ghost" onclick={() => (renaming = false)}>{m.cancel()}</button>
				</form>
			{:else}
				<div class="title-row">
					<h3 class="name" class:unnamed={!rabbit.name} id="{id}-name">{name}</h3>
					{#if rabbit.mac}
						<button class="icon-btn" aria-label={m.name_edit({ name })} onclick={startRename}><Icon name="pencil" size={16} /></button>
					{/if}
				</div>
			{/if}
			<p class="who mono">{rabbit.mac ? formatMac(rabbit.mac) : m.rabbit_no_mac()}</p>
			<p class="state" class:warn={condition.warn}>
				<span class="dot" class:on={awake} aria-hidden="true"></span>{condition.text}
			</p>
		</div>
		<div class="end">
			<span class="chip mono">{rabbit.ip}</span>
			{#if rabbit.version}<span class="chip accent">{m.rabbit_version({ version: rabbit.version })}</span>{/if}
		</div>
	</div>

	<dl class="facts">
		{#each facts as f (f.label)}
			<div class="fact">
				<dt><Icon name={f.icon} size={16} />{f.label}</dt>
				<dd>{f.value ?? m.fact_unknown()}</dd>
			</div>
		{/each}
	</dl>

	{#if rabbit.mac}
		<div class="gestures">
			{#each quick as q (q.key)}
				<button class="btn" disabled={busy !== null} onclick={() => ctl(q.key, q.cmd)}>
					<Icon name={q.icon} busy={busy === q.key} />{q.label()}
				</button>
			{/each}
			<ConfirmDialog
				title={m.reboot_title({ name })}
				body={m.reboot_body()}
				confirm={m.action_reboot()}
				onconfirm={() => ctl('reboot', 'reboot')}
			>
				{#snippet trigger(props)}
					<button {...props} class="btn danger" disabled={busy !== null}>
						<Icon name="power" busy={busy === 'reboot'} />{m.action_reboot()}
					</button>
				{/snippet}
			</ConfirmDialog>
		</div>

		<Accordion.Root type="multiple" class="more">
			<Accordion.Item value="dance" class="more-item">
				<Accordion.Header level={4}>
					<Accordion.Trigger class="more-trigger"><Icon name="music" />{m.section_dance()}<Icon name="chevron-down" class="chev" /></Accordion.Trigger>
				</Accordion.Header>
				<Accordion.Content class="more-body">
					{#if chorOptions.length}
						<div class="inline">
							<Select label={m.dance_library()} value={chor} options={chorOptions} placeholder={m.dance_pick()} onchange={(v) => (chor = v)} />
							<button class="btn primary" disabled={!chor || busy !== null} onclick={() => ctl('chor', `chor ${chor}`)}>
								<Icon name="play" busy={busy === 'chor'} />{m.action_dance()}
							</button>
						</div>
					{:else}
						<p class="hint">{m.dance_library_empty()}</p>
					{/if}
					<div class="field">
						<label for="{id}-cdl">{m.dance_compose()}</label>
						<textarea id="{id}-cdl" class="mono" bind:value={cdl} placeholder="fps,t,led,4,124,92,255,t,motor,0,90,0,0" aria-describedby="{id}-cdl-help"></textarea>
						<span class="help" id="{id}-cdl-help">{m.dance_compose_help()}</span>
					</div>
					<div class="actions end">
						<button class="btn primary" disabled={!cdl.trim() || busy !== null} onclick={compose}>
							<Icon name="sparkles" busy={busy === 'compose'} />{m.action_compose()}
						</button>
					</div>
				</Accordion.Content>
			</Accordion.Item>

			<Accordion.Item value="look" class="more-item">
				<Accordion.Header level={4}>
					<Accordion.Trigger class="more-trigger"><Icon name="palette" />{m.section_look()}<Icon name="chevron-down" class="chev" /></Accordion.Trigger>
				</Accordion.Header>
				<Accordion.Content class="more-body">
					<div role="group" aria-labelledby="{id}-color" class="block">
						<p class="label" id="{id}-color">{m.look_color()}</p>
						<div class="swatches">
							{#each colors as c (c.hex)}
								<button
									class="swatch"
									class:off={c.hex === '000000'}
									data-hex={c.hex}
									aria-label={c.label()}
									title={c.label()}
									disabled={busy !== null}
									onclick={() => ctl(`color-${c.hex}`, `color ${c.hex}`)}
								></button>
							{/each}
						</div>
					</div>
					<div role="group" aria-labelledby="{id}-ears" class="block">
						<p class="label" id="{id}-ears">{m.fact_ears()}</p>
						{#each [{ side: m.ear_left(), get: () => left, set: (v: number) => (left = v) }, { side: m.ear_right(), get: () => right, set: (v: number) => (right = v) }] as ear (ear.side)}
							<div class="ear">
								<span class="ear-side">{ear.side}</span>
								<Slider.Root
									type="single"
									min={0}
									max={16}
									step={1}
									value={ear.get()}
									onValueChange={(v) => {
										earsTouched = true;
										ear.set(v);
									}}
									class="slider"
								>
									<span class="track"><Slider.Range class="range" /></span>
									<Slider.Thumb index={0} class="thumb" aria-label={m.ear_position({ side: ear.side })} />
								</Slider.Root>
								<span class="ear-value">{ear.get()}</span>
							</div>
						{/each}
						<div class="actions end">
							<button class="btn primary" disabled={busy !== null} onclick={() => ctl('ears', `ears ${left} ${right}`).then(() => (earsTouched = false))}>
								<Icon name="ear" busy={busy === 'ears'} />{m.action_ears()}
							</button>
						</div>
					</div>
				</Accordion.Content>
			</Accordion.Item>

			{#if say}
				<Accordion.Item value="say" class="more-item">
					<Accordion.Header level={4}>
						<Accordion.Trigger class="more-trigger"><Icon name="message-circle" />{m.section_say()}<Icon name="chevron-down" class="chev" /></Accordion.Trigger>
					</Accordion.Header>
					<Accordion.Content class="more-body">
						<div class="field">
							<label for="{id}-say">{m.say_label()}</label>
							<textarea id="{id}-say" bind:value={sentence} maxlength="300" placeholder={m.say_placeholder()}></textarea>
							<span class="help">{m.say_help({ count: 300 - sentence.length })}</span>
						</div>
						<div class="actions end">
							<button class="btn primary" disabled={!sentence.trim() || busy !== null} onclick={speak}>
								<Icon name="message-circle" busy={busy === 'say'} />{m.action_say()}
							</button>
						</div>
					</Accordion.Content>
				</Accordion.Item>
			{/if}
		</Accordion.Root>
	{:else}
		<p class="hint">{m.rabbit_no_mac_hint()}</p>
	{/if}
</article>

<style>
	.head { display: flex; align-items: flex-start; gap: var(--s-3); min-width: 0; flex-wrap: wrap; }
	.glyph { flex: none; display: grid; place-items: center; width: calc(var(--tile-icon) + 8px); height: calc(var(--tile-icon) + 8px); border-radius: var(--radius-l); background: var(--ground-raised); }
	.glyph.awake { background: var(--accent-wash); }
	.text { min-width: 0; flex: 1; display: grid; gap: 2px; }
	.name { font: var(--t-group); margin: 0; }
	.name.unnamed { color: var(--ink-muted); font-style: italic; }
	.title-row { display: flex; align-items: center; gap: var(--s-1); }
	.rename { display: flex; flex-wrap: wrap; gap: var(--s-2); align-items: center; }
	.rename input { flex: 1 1 8rem; min-width: 0; }
	.who { margin: 0; color: var(--ink-muted); }
	.state { margin: 0; font: var(--t-secondary); color: var(--ink-muted); display: inline-flex; align-items: center; gap: var(--s-2); }
	.condition.warn { color: var(--warn-text); }
	/* awake: a filled disc; otherwise an empty ring (two shapes, never colour alone) */
	.dot { width: 8px; height: 8px; border-radius: var(--radius-pill); border: 1.5px solid currentColor; flex: none; }
	.dot.on { background: var(--status-ok); border-color: var(--status-ok); }
	.end { display: flex; flex-direction: column; align-items: flex-end; gap: var(--s-1); }
	@media (max-width: 599px) {
		.end { flex-direction: row; width: 100%; align-items: center; }
	}

	/* four gestures in a row, two by two when the tile is narrow (no word cut at 320 px) */
	.gestures { display: grid; grid-template-columns: repeat(auto-fit, minmax(7.5rem, 1fr)); gap: var(--s-2); }
	.gestures :global(.btn) { justify-content: center; min-height: var(--control-h); }
	.facts { display: grid; grid-template-columns: repeat(auto-fill, minmax(9.5rem, 1fr)); gap: var(--s-2) var(--s-4); margin: 0; }
	.fact { display: grid; gap: 2px; min-width: 0; }
	.fact dt { display: inline-flex; align-items: center; gap: 6px; font: var(--t-meta); color: var(--ink-muted); }
	.fact dd { margin: 0; font: var(--t-body); font-variant-numeric: tabular-nums; overflow-wrap: anywhere; }

	:global(.more) { display: grid; border-top: 1px solid var(--line); }
	:global(.more-item) { border-bottom: 1px solid var(--line); }
	:global(.more-item:last-child) { border-bottom: 0; }
	:global(.more-trigger) {
		display: flex; align-items: center; gap: var(--s-2); width: 100%; min-height: var(--control-h); padding: 0 var(--s-1);
		border: 0; background: none; font: var(--t-label); color: var(--ink); cursor: pointer; text-align: left; border-radius: var(--radius);
	}
	:global(.more-trigger:hover) { background: var(--ground-raised); }
	:global(.more-trigger .chev) { margin-left: auto; transition: transform 0.2s; color: var(--ink-muted); }
	:global(.more-trigger[data-state='open'] .chev) { transform: rotate(180deg); }
	:global(.more-body) { display: grid; gap: var(--s-3); padding: var(--s-2) var(--s-1) var(--s-4); }

	.inline { display: grid; grid-template-columns: minmax(0, 1fr) auto; gap: var(--s-2); align-items: end; }
	.inline .btn { min-height: var(--control-h); }

	.swatches { display: flex; flex-wrap: wrap; gap: var(--s-2); }
	.swatch {
		width: var(--control-h-xs); height: var(--control-h-xs); border-radius: var(--radius-pill); cursor: pointer;
		border: 2px solid var(--surface); box-shadow: 0 0 0 1px var(--line);
	}
	.swatch:hover { box-shadow: 0 0 0 2px var(--accent); }
	.swatch[disabled] { opacity: 0.55; cursor: not-allowed; }
	.swatch[data-hex='7c5cff'] { background: #7c5cff; }
	.swatch[data-hex='2b7bff'] { background: #2b7bff; }
	.swatch[data-hex='2fd36b'] { background: #2fd36b; }
	.swatch[data-hex='ffd23f'] { background: #ffd23f; }
	.swatch[data-hex='ff8a1f'] { background: #ff8a1f; }
	.swatch[data-hex='ff4fa3'] { background: #ff4fa3; }
	.swatch[data-hex='ffffff'] { background: #ffffff; }
	/* off: a ring with a slash, not a black disc that reads as a colour */
	.swatch.off { background: linear-gradient(135deg, transparent 45%, var(--ink-muted) 45% 55%, transparent 55%), var(--surface); }

	.ear { display: grid; grid-template-columns: 4.5rem minmax(0, 1fr) 2ch; align-items: center; gap: var(--s-3); min-height: var(--control-h); }
	.ear-side { font: var(--t-secondary); color: var(--ink-muted); }
	.ear-value { font: var(--t-label); font-variant-numeric: tabular-nums; text-align: right; }
	:global(.slider) { position: relative; display: flex; align-items: center; width: 100%; height: var(--control-h); touch-action: none; }
	.track { position: relative; flex: 1; height: 6px; border-radius: var(--radius-pill); background: var(--ground-raised); overflow: hidden; }
	:global(.slider .range) { position: absolute; height: 100%; background: var(--accent); }
	:global(.slider .thumb) {
		display: block; width: 22px; height: 22px; border-radius: var(--radius-pill); background: var(--surface);
		border: 2px solid var(--accent); box-shadow: var(--shadow); cursor: grab;
	}
</style>
