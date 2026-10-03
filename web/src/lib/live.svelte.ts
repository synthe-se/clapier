// The burrow's state, asked every few seconds while the page is looked at (a rabbit pulses
// every 2 s), never while it is hidden. A missed answer keeps the last state on screen and
// says so; the next one that arrives clears it.

import { api, type Status } from '#lib/api.ts';

export const EVERY_MS = 3000;

class Live {
	status = $state<Status | null>(null);
	/** The last ask went unanswered. */
	lost = $state(false);
	#timer: ReturnType<typeof setTimeout> | undefined;

	async refresh() {
		clearTimeout(this.#timer);
		try {
			this.status = await api.status();
			this.lost = false;
		} catch {
			this.lost = true;
		}
		if (document.visibilityState === 'visible') this.#timer = setTimeout(() => this.refresh(), EVERY_MS);
	}

	/** Start asking; returns the stop. */
	start(): () => void {
		const seen = () => {
			if (document.visibilityState === 'visible') void this.refresh();
			else clearTimeout(this.#timer);
		};
		document.addEventListener('visibilitychange', seen);
		void this.refresh();
		return () => {
			document.removeEventListener('visibilitychange', seen);
			clearTimeout(this.#timer);
		};
	}
}

export const live = new Live();
