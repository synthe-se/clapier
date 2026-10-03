// A refusal in the person's words: the burrow sends a code, never a sentence.

import { m } from '#lib/paraglide/messages.js';
import { ApiError } from '#lib/api.ts';

const WORDS: Record<string, () => string> = {
	unreachable: m.error_unreachable,
	bad_ip: m.error_bad_ip,
	bad_mac: m.error_bad_mac,
	unknown_rabbit: m.error_unknown_rabbit,
	no_reply: m.error_no_reply,
	cmd_empty: m.error_cmd_empty,
	cmd_not_allowed: m.error_cmd_not_allowed,
	vol_floor: m.error_vol_floor,
	vol_number: m.error_vol_number,
	ears_range: m.error_ears_range,
	no_overlay: m.error_no_overlay,
	install_failed: m.error_install_failed,
	say_off: m.error_say_off,
	say_length: m.error_say_length,
	say_failed: m.error_say_failed,
	name_length: m.error_name_length,
	name_chars: m.error_name_chars,
	name_failed: m.error_name_failed
};

export function errorText(error: unknown): string {
	if (error instanceof ApiError) {
		if (error.code === 'chor_rejected') return m.error_chor_rejected({ reason: error.detail ?? '' });
		const words = WORDS[error.code];
		if (words) return words();
	}
	return m.error_unknown();
}
