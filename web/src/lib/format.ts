// Durations, ages, sizes and rabbits in the reader's words. Pure: the locale is passed in.

/** The `units` largest units of a duration: « 3 j 4 h », « 12 min 5 s » (Intl.DurationFormat, narrow). */
export function duration(seconds: number, locale: string, units = 2): string {
	const s = Math.max(0, Math.floor(seconds));
	const parts: [Intl.DurationFormatUnit, number][] = [
		['days', Math.floor(s / 86400)],
		['hours', Math.floor((s % 86400) / 3600)],
		['minutes', Math.floor((s % 3600) / 60)],
		['seconds', s % 60]
	];
	const first = parts.findIndex(([, v]) => v > 0);
	if (first === -1) return new Intl.DurationFormat(locale, { style: 'narrow', secondsDisplay: 'always' }).format({ seconds: 0 });
	const kept = parts.slice(first, first + units).filter(([, v], i) => i === 0 || v > 0);
	return new Intl.DurationFormat(locale, { style: 'narrow' }).format(Object.fromEntries(kept));
}

/** A byte count: « 812 o », « 14,2 ko » (units from Intl, the reader's decimals). */
export function bytes(n: number, locale: string): string {
	const [value, unit] = n < 1000 ? [n, 'byte'] : n < 1e6 ? [n / 1000, 'kilobyte'] : [n / 1e6, 'megabyte'];
	return new Intl.NumberFormat(locale, { style: 'unit', unit, unitDisplay: 'short', maximumFractionDigits: n < 1000 ? 0 : 1 }).format(value);
}

/** « 0019db9c2815 » → « 00:19:db:9c:28:15 ». */
export function mac(raw: string): string {
	return raw.match(/../g)?.join(':') ?? raw;
}

/** An ear position (0..16 around the turn, 0 up) as degrees, mirrored for the left ear. */
export function earAngle(position: number, left: boolean): number {
	const deg = (position % 17) * (360 / 17);
	return left ? -deg : deg;
}

/** How an ear stands, in words the tile can say: up, tilted, flat (sideways), down. */
export function earPose(position: number): 'up' | 'tilted' | 'flat' | 'down' {
	const deg = Math.abs(earAngle(position, false)) % 360;
	const fromUp = Math.min(deg, 360 - deg);
	return fromUp < 10 ? 'up' : fromUp < 60 ? 'tilted' : fromUp <= 120 ? 'flat' : 'down';
}

/** The rabbit's `ears=L,R` as two positions, or null. */
export function ears(raw: string | null): [number, number] | null {
	const p = raw?.split(',').map(Number);
	return p && p.length === 2 && p.every((n) => Number.isInteger(n) && n >= 0 && n <= 16) ? [p[0], p[1]] : null;
}

/** A position counted from another one, around the 17 holes of the turn. */
export const earTurn = (position: number, from: number): number => (((position - from) % 17) + 17) % 17;

/** A rabbit is awake while its 2 s pulse keeps coming (a few missed ones forgiven). */
export const AWAKE_S = 10;
