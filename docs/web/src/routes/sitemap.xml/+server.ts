import { SITE_URL } from '$lib/constants/site';
import { FALLBACK_LOCALE, getFlatOrder } from '$lib/content/docs';
import { locales, localizeHref } from '$lib/paraglide/runtime';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async () => {
	const lastmod = new Date().toISOString().split('T')[0];

	const paths = ['/', ...getFlatOrder(FALLBACK_LOCALE).map((doc) => `/docs/${doc.slug}`)];

	const urls = paths
		.map((path) => {
			const alternates = locales
				.map(
					(locale) => `
        <xhtml:link rel="alternate" hreflang="${locale}" href="${SITE_URL}${localizeHref(path, { locale })}" />`
				)
				.join('');

			return `
    <url>
        <loc>${SITE_URL}${localizeHref(path, { locale: FALLBACK_LOCALE })}</loc>
        <lastmod>${lastmod}</lastmod>
        <changefreq>weekly</changefreq>
        <priority>${path === '/' ? '1.0' : '0.8'}</priority>${alternates}
    </url>`;
		})
		.join('');

	const sitemap = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"
        xmlns:xhtml="http://www.w3.org/1999/xhtml">${urls}
</urlset>`;

	return new Response(sitemap, {
		headers: {
			'Content-Type': 'application/xml',
			'Cache-Control': 'max-age=0, s-maxage=3600'
		}
	});
};
