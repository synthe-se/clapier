<script lang="ts">
	// A sticky 56 px header whose inner row is the page container: the brand, whether the burrow
	// answers, and the display settings (language, theme) in one panel.
	import { Popover } from 'bits-ui';
	import { m } from '#lib/paraglide/messages.js';
	import { ui, type Theme } from '#lib/ui.svelte.ts';
	import { locale, localeName, locales, switchLocale } from '#lib/i18n.svelte.ts';
	import { live } from '#lib/live.svelte.ts';
	import Brand from './Brand.svelte';
	import Icon from './Icon.svelte';

	const themes: { value: Theme; label: () => string }[] = [
		{ value: 'system', label: m.theme_system },
		{ value: 'light', label: m.theme_light },
		{ value: 'dark', label: m.theme_dark }
	];
</script>

<header class="top">
	<div class="page bar">
		<Brand />
		<div class="grow"></div>
		{#if live.status}
			<span class="pulse" class:lost={live.lost}>
				<span class="dot" aria-hidden="true"></span>
				{live.lost ? m.live_lost() : m.live_on()}
			</span>
		{/if}
		<Popover.Root>
			<Popover.Trigger class="icon-btn" aria-label={m.settings_open()}>
				<Icon name="settings-2" size={20} />
			</Popover.Trigger>
			<Popover.Portal>
				<Popover.Content class="panel" sideOffset={6} align="end" role="dialog" aria-labelledby="settings-title">
					<h2 id="settings-title" class="settings-title">{m.settings_title()}</h2>
					<div role="group" aria-labelledby="lang-label">
						<h3 class="label" id="lang-label">{m.language_label()}</h3>
						<div class="row">
							{#each locales as l (l)}
								<button class="btn" lang={l} aria-pressed={locale() === l} onclick={() => switchLocale(l)}>{localeName(l)}</button>
							{/each}
						</div>
					</div>
					<div role="group" aria-labelledby="theme-label">
						<h3 class="label" id="theme-label">{m.theme_label()}</h3>
						<div class="row">
							{#each themes as t (t.value)}
								<button class="btn" aria-pressed={ui.theme === t.value} onclick={() => ui.setTheme(t.value)}>{t.label()}</button>
							{/each}
						</div>
					</div>
				</Popover.Content>
			</Popover.Portal>
		</Popover.Root>
	</div>
</header>

<style>
	.top { position: sticky; top: 0; z-index: 20; border-bottom: 1px solid var(--line); background: var(--ground); }
	.bar { display: flex; align-items: center; gap: var(--s-3); min-height: var(--header-h); }
	.grow { flex: 1; }
	.pulse { display: inline-flex; align-items: center; gap: var(--s-2); font: var(--t-meta); color: var(--ink-muted); white-space: nowrap; }
	.pulse.lost { color: var(--warn-text); }
	/* live: a filled disc that breathes; lost: an empty ring (two shapes, never colour alone) */
	.dot { width: 10px; height: 10px; border-radius: var(--radius-pill); background: var(--status-ok); animation: breathe 2s ease-in-out infinite; }
	.lost .dot { background: none; border: 2px solid var(--status-degraded); animation: none; }
	@keyframes breathe { 50% { opacity: 0.35; } }
	:global(.panel .settings-title) { font: var(--t-group); margin: 0; }
	:global(.panel .label) { font-family: var(--font-text); font-size: 0.85rem; font-weight: 600; margin-bottom: 6px; }
</style>
