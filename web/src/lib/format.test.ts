import { describe, expect, it } from 'vitest';
import { bytes, earPose, duration, earAngle, earTurn, ears, mac } from './format.ts';

describe('format', () => {
	it('keeps the two largest units of a duration', () => {
		expect(duration(0, 'en')).toBe('0s');
		expect(duration(42, 'en')).toBe('42s');
		expect(duration(3 * 86400 + 4 * 3600 + 5, 'en')).toBe('3d 4h');
		expect(duration(3600 + 5, 'en')).toBe('1h');
		expect(duration(16487, 'fr')).toMatch(/^4\s?h 34\s?min$/);
	});

	it('can keep a single unit', () => {
		expect(duration(7300, 'en', 1)).toBe('2h');
	});

	it('sizes in the reader’s units', () => {
		expect(bytes(812, 'en')).toBe('812 byte');
		expect(bytes(14_200, 'en')).toBe('14.2 kB');
	});

	it('writes a MAC with colons', () => {
		expect(mac('0019db9c2815')).toBe('00:19:db:9c:28:15');
	});

	it('says how an ear stands', () => {
		expect([0, 2, 4, 8, 13, 16].map(earPose)).toEqual(['up', 'tilted', 'flat', 'down', 'flat', 'tilted']);
	});

	it('counts an ear from where its up is', () => {
		expect(earTurn(0, 0)).toBe(0);
		expect(earTurn(3, 5)).toBe(15);
		expect(earTurn(5, 3)).toBe(2);
		// and back: what to ask the rabbit for a position counted from its up
		expect(earTurn(15, -5)).toBe(3);
	});

	it('reads the ears, and only sane ones', () => {
		expect(ears('0,4')).toEqual([0, 4]);
		expect(ears('0,17')).toBeNull();
		expect(ears('x')).toBeNull();
		expect(ears(null)).toBeNull();
		expect(earAngle(0, true)).toBe(-0);
		expect(earAngle(17 / 4, false)).toBeCloseTo(90);
	});
});
