<script lang="ts">
	import { m } from '#lib/paraglide/messages.js';
</script>

<a class="brand" href={import.meta.env.BASE_URL} aria-label={m.brand_name()}>
	<img src="{import.meta.env.BASE_URL}brand.svg" alt="" width="28" height="28" />
	<span class="name" aria-hidden="true">{m.brand_name()}</span>
</a>

<style>
	.brand { display: inline-flex; align-items: center; gap: var(--s-2); min-height: var(--control-h); color: var(--ink); text-decoration: none; }
	/* the name only, in Borel (as Maison and Ariane): never a title, never body text */
	.name { font-family: var(--font-brand); font-size: 1.35rem; line-height: 1; padding-top: 0.3em; }
</style>
