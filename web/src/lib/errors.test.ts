import { describe, expect, it } from 'vitest';
import { ApiError } from './api.ts';
import { errorText } from './errors.ts';
import { m } from './paraglide/messages.js';

describe('errorText', () => {
	it('words every code the burrow sends', () => {
		expect(errorText(new ApiError('vol_floor'))).toBe(m.error_vol_floor());
		expect(errorText(new ApiError('unknown_rabbit'))).toBe(m.error_unknown_rabbit());
	});

	it('shows the encoder’s reason as is', () => {
		expect(errorText(new ApiError('chor_rejected', 'bad fps'))).toContain('bad fps');
	});

	it('falls back for what it does not know', () => {
		expect(errorText(new ApiError('nope'))).toBe(m.error_unknown());
		expect(errorText(new Error('x'))).toBe(m.error_unknown());
	});
});
