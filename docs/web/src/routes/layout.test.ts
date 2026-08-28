import { describe, expect, it } from 'vitest';
import { prerender } from './+layout';

describe('+layout', () => {
	it('prerenders the whole site', () => {
		expect(prerender).toBe(true);
	});
});
