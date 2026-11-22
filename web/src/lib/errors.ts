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
	name_length: m.error_name_length,
	name_chars: m.error_name_chars,
	name_failed: m.error_name_failed,
	fw_shape: m.error_fw_shape,
	fw_size: m.error_fw_size,
	fw_vectors: m.error_fw_vectors,
	fw_name: m.error_file_name,
	fw_missing: m.error_fw_missing,
	fw_keep_failed: m.error_fw_keep_failed,
	sound_shape: m.error_sound_shape,
	sound_name: m.error_file_name,
	sound_size: m.error_sound_size,
	sound_keep_failed: m.error_sound_keep_failed
};

export function errorText(error: unknown): string {
	if (error instanceof ApiError) {
		if (error.code === 'chor_rejected') return m.error_chor_rejected({ reason: error.detail ?? '' });
		const words = WORDS[error.code];
		if (words) return words();
	}
	return m.error_unknown();
}
