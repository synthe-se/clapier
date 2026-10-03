// Locale switching without reload, and the reader's formats bound to it.

import { m } from '#lib/paraglide/messages.js';
import {
	getLocale as resolveLocale,
	locales,
	overwriteGetLocale,
	setLocale as persistLocale,
	type Locale
} from '#lib/paraglide/runtime.js';
import * as f from '#lib/format.ts';

const current = $state<{ locale: Locale }>({ locale: resolveLocale() });
overwriteGetLocale(() => current.locale);
if (typeof document !== 'undefined') document.documentElement.lang = current.locale;

export { locales };
export type { Locale };

export const locale = (): Locale => current.locale;

export function switchLocale(next: Locale) {
	if (next === current.locale) return;
	persistLocale(next, { reload: false });
	current.locale = next;
	document.documentElement.lang = next;
}

/** A language named in itself, from its own message file. */
export function localeName(l: Locale): string {
	return m.locale_name({}, { locale: l });
}

export const duration = (s: number) => f.duration(s, current.locale);
/** How long ago, in one unit: « il y a 3 min », « 3m ago ». */
export const ago = (s: number) => m.time_ago({ duration: f.duration(s, current.locale, 1) });
export const bytes = (n: number) => f.bytes(n, current.locale);
export const num = (n: number) => new Intl.NumberFormat(current.locale).format(n);
