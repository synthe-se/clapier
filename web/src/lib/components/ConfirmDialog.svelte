<script lang="ts">
	// A gesture that cannot be taken back (a reboot), asked once in a dialog (APG alertdialog,
	// through Bits UI). The trigger is drawn by the caller.
	import type { Snippet } from 'svelte';
	import { AlertDialog } from 'bits-ui';
	import { m } from '#lib/paraglide/messages.js';

	interface Props {
		title: string;
		body: string;
		confirm: string;
		onconfirm: () => void;
		trigger: Snippet<[Record<string, unknown>]>;
	}
	let { title, body, confirm, onconfirm, trigger }: Props = $props();
	let open = $state(false);
</script>

<AlertDialog.Root bind:open>
	<AlertDialog.Trigger>
		{#snippet child({ props })}{@render trigger(props)}{/snippet}
	</AlertDialog.Trigger>
	<AlertDialog.Portal>
		<AlertDialog.Overlay class="overlay" />
		<AlertDialog.Content class="modal">
			<div class="dlg-head"><AlertDialog.Title class="grow dlg-title">{title}</AlertDialog.Title></div>
			<div class="dlg-body">
				<AlertDialog.Description>{body}</AlertDialog.Description>
				<div class="actions end">
					<AlertDialog.Cancel class="btn">{m.cancel()}</AlertDialog.Cancel>
					<AlertDialog.Action
						class="btn primary"
						onclick={() => {
							open = false;
							onconfirm();
						}}>{confirm}</AlertDialog.Action
					>
				</div>
			</div>
		</AlertDialog.Content>
	</AlertDialog.Portal>
</AlertDialog.Root>

<style>
	:global(.dlg-title) { font: var(--t-group); margin: 0; }
	:global(.dlg-body p) { margin: 0; }
</style>
