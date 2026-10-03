import { paraglideVitePlugin } from '@inlang/paraglide-js';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { defineConfig } from 'vitest/config';

// One screen, no routes: plain Svelte. The build is embedded into the clapier binary
// (crates/clapier-pages/build.rs): the page at /, its files and API under /_clapier/.
export default defineConfig({
	base: '/_clapier/',
	plugins: [
		svelte(),
		paraglideVitePlugin({
			project: '../i18n/project.inlang',
			outdir: './src/lib/paraglide',
			strategy: ['localStorage', 'preferredLanguage', 'baseLocale'],
			localStorageKey: 'clapier-locale',
			emitTsDeclarations: true
		})
	],
	build: { target: 'es2023', assetsInlineLimit: 0 },
	test: {
		expect: { requireAssertions: true },
		restoreMocks: true,
		unstubGlobals: true,
		environment: 'node',
		include: ['src/**/*.test.ts']
	},
	server: {
		// `just serve` on :8080; CLAPIER=http://192.168.1.103 to look at the Pi's burrow
		proxy: { '/_clapier/api': { target: process.env.CLAPIER ?? 'http://127.0.0.1:8080', changeOrigin: true } }
	}
});
