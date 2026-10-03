<script lang="ts" generics="T extends string">
	// A labelled choice among a few values (Bits UI Select, APG listbox in a popup).
	import { Select } from 'bits-ui';
	import Icon from './Icon.svelte';

	interface Props {
		label: string;
		value: T;
		options: readonly { value: T; label: string }[];
		onchange: (v: T) => void;
		disabled?: boolean;
		placeholder?: string;
		hideLabel?: boolean;
	}
	let { label, value, options, onchange, disabled = false, placeholder, hideLabel = false }: Props = $props();
	const id = $props.id();
	const shown = $derived(options.find((o) => o.value === value)?.label ?? placeholder ?? '');
</script>

<div class="field">
	<span class="label" class:sr-only={hideLabel} id="{id}-label">{label}</span>
	<Select.Root type="single" {value} {disabled} items={options.map((o) => ({ ...o }))} onValueChange={(v) => onchange(v as T)}>
		<Select.Trigger class="choice-trigger" aria-labelledby="{id}-label {id}-value">
			<span id="{id}-value" class:muted={!value}>{shown}</span>
			<Icon name="chevron-down" />
		</Select.Trigger>
		<Select.Portal>
			<Select.Content class="select-content choice-content" sideOffset={4}>
				{#each options as o (o.value)}
					<Select.Item class="select-item" value={o.value} label={o.label}>
						{#snippet children({ selected })}
							<span class="mark">{#if selected}<Icon name="check" />{/if}</span>{o.label}
						{/snippet}
					</Select.Item>
				{/each}
			</Select.Content>
		</Select.Portal>
	</Select.Root>
</div>

<style>
	:global(.choice-trigger) {
		display: flex; align-items: center; justify-content: space-between; gap: var(--s-2); width: 100%;
		min-height: var(--control-h); padding: 0 var(--s-3); border: 1px solid var(--line); border-radius: var(--radius);
		background: var(--ground); color: var(--ink); font: var(--t-body); cursor: pointer; text-align: left; min-width: 0;
	}
	:global(.choice-trigger > span) { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	:global(.choice-trigger[data-disabled]) { opacity: 0.55; cursor: not-allowed; }
	:global(.choice-content) { min-width: var(--bits-floating-anchor-width); max-height: var(--bits-floating-available-height); overflow-y: auto; }
	.mark { display: inline-grid; width: 18px; color: var(--accent); }
</style>
