import { describe, expect, it } from 'vitest';
import { MODE_STORAGE_KEY, THEME_STORAGE_KEY, THEMES } from './themes';

describe('themes constants', () => {
	it('defines a light and dark variant for every theme family', () => {
		const families = Object.keys(THEMES);
		expect(families).toEqual(['catppuccin', 'nord', 'dracula', 'tokyo-night']);

		for (const family of families) {
			const variants = THEMES[family as keyof typeof THEMES];
			expect(variants).toHaveProperty('light');
			expect(variants).toHaveProperty('dark');
			expect(typeof variants.light).toBe('string');
			expect(typeof variants.dark).toBe('string');
		}
	});

	it('exposes distinct localStorage keys for theme and mode', () => {
		expect(THEME_STORAGE_KEY).toBe('acerola-docs:theme');
		expect(MODE_STORAGE_KEY).toBe('acerola-docs:mode');
		expect(THEME_STORAGE_KEY).not.toBe(MODE_STORAGE_KEY);
	});
});
