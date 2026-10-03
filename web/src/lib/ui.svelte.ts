// What the interface remembers about itself: the theme, the toasts, and the two live regions
// that say each outcome once (App mounts them, only their text changes).

import { errorText } from '#lib/errors.ts';

export type Theme = 'system' | 'light' | 'dark';
const THEME_KEY = 'clapier-theme'; // public/theme.js reads it before first paint

type Toast = { id: number; text: string; warn: boolean };

function storedTheme(): Theme {
	try {
		const t = localStorage.getItem(THEME_KEY);
		return t === 'light' || t === 'dark' ? t : 'system';
	} catch {
		return 'system';
	}
}

class Ui {
	theme = $state<Theme>(storedTheme());
	toasts = $state<Toast[]>([]);
	polite = $state('');
	assertive = $state('');
	#next = 0;
	#timers = new Map<number, ReturnType<typeof setTimeout>>();

	setTheme(theme: Theme) {
		this.theme = theme;
		const root = document.documentElement;
		if (theme === 'system') delete root.dataset.theme;
		else root.dataset.theme = theme;
		try {
			if (theme === 'system') localStorage.removeItem(THEME_KEY);
			else localStorage.setItem(THEME_KEY, theme);
		} catch {
			// private mode: the choice lasts for the visit
		}
	}

	/** A short message under the fingers, read once. Errors stay until dismissed. */
	toast(text: string, warn = false) {
		for (const t of this.toasts) if (t.text === text && t.warn === warn) this.dismiss(t.id);
		const id = this.#next++;
		this.toasts.push({ id, text, warn });
		const region = warn ? 'assertive' : 'polite';
		this[region] = '';
		queueMicrotask(() => (this[region] = text));
		if (!warn) this.resume(id);
	}

	/** A gesture failed: said in plain words (errors.ts), shown until read. */
	fail(error: unknown) {
		this.toast(errorText(error), true);
	}

	dismiss(id: number) {
		clearTimeout(this.#timers.get(id));
		this.#timers.delete(id);
		this.toasts = this.toasts.filter((t) => t.id !== id);
	}

	/** Hover or focus holds a toast; leaving lets it go after 5 s (WCAG 2.2.1). */
	pause(id: number) {
		clearTimeout(this.#timers.get(id));
	}

	resume(id: number) {
		if (this.toasts.find((t) => t.id === id)?.warn) return;
		this.#timers.set(
			id,
			setTimeout(() => this.dismiss(id), 5000)
		);
	}
}

export const ui = new Ui();
