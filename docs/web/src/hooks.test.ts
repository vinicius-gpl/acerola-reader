import { describe, expect, it } from 'vitest';
import { reroute } from './hooks';

describe('reroute', () => {
	it('keeps the path unchanged for the default locale', () => {
		const result = reroute({ url: new URL('http://localhost/docs/getting-started') });

		expect(result).toBe('/docs/getting-started');
	});

	it('strips the locale prefix from a localized path', () => {
		const result = reroute({ url: new URL('http://localhost/en/docs/getting-started') });

		expect(result).toBe('/docs/getting-started');
	});
});
