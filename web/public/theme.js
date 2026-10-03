// The chosen theme before first paint, from a file (the policy allows no inline script).
try {
	const t = localStorage.getItem('clapier-theme');
	if (t === 'light' || t === 'dark') document.documentElement.dataset.theme = t;
} catch {}
