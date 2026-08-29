import { describe, expect, it, vi } from 'vitest';

vi.mock('$lib/paraglide/server', () => ({
	paraglideMiddleware: (
		request: Request,
		callback: (arg: { request: Request; locale: string }) => unknown
	) => callback({ request, locale: 'en' })
}));

vi.mock('$lib/paraglide/runtime', () => ({
	getTextDirection: (locale: string) => (locale === 'ar' ? 'rtl' : 'ltr')
}));

import { handle } from './hooks.server';

describe('handle (paraglide middleware)', () => {
	it('replaces the lang and dir placeholders in the resolved HTML', async () => {
		const resolve = vi.fn(
			async (
				event: { request: Request },
				options: { transformPageChunk: (arg: { html: string }) => string }
			) =>
				new Response(
					options.transformPageChunk({
						html: '<html lang="%paraglide.lang%" dir="%paraglide.dir%"></html>'
					})
				)
		);

		const event = { request: new Request('http://localhost/') } as unknown as Parameters<
			typeof handle
		>[0]['event'];

		const response = await handle({ event, resolve } as unknown as Parameters<typeof handle>[0]);
		const html = await (response as Response).text();

		expect(html).toBe('<html lang="en" dir="ltr"></html>');
		expect(resolve).toHaveBeenCalled();
	});
});
