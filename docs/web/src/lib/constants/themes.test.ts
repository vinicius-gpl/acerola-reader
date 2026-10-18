import { describe, expect, it } from 'vitest';
import { MODE_STORAGE_KEY } from './themes';

describe('themes constants', () => {
	it('exposes the localStorage key for the light/dark mode', () => {
		expect(MODE_STORAGE_KEY).toBe('acerola-docs:mode');
	});
});
