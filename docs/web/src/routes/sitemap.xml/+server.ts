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
					(locale) =>
						`\t\t<xhtml:link rel="alternate" hreflang="${locale}" href="${SITE_URL}${localizeHref(path, { locale })}" />`
				)
				.join('\n');

			return `\t<url>
\t\t<loc>${SITE_URL}${localizeHref(path, { locale: FALLBACK_LOCALE })}</loc>
\t\t<lastmod>${lastmod}</lastmod>
\t\t<changefreq>weekly</changefreq>
\t\t<priority>${path === '/' ? '1.0' : '0.8'}</priority>
${alternates}
\t</url>`;
		})
		.join('\n');

	const sitemap = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"
        xmlns:xhtml="http://www.w3.org/1999/xhtml">
${urls}
</urlset>`;

	return new Response(sitemap, {
		headers: {
			'Content-Type': 'application/xml',
			'Cache-Control': 'max-age=0, s-maxage=3600'
		}
	});
};
