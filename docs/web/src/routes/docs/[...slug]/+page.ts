import { error } from '@sveltejs/kit';
import { getDoc, getPrevNext } from '$lib/content/docs';
import { getLocale } from '$lib/paraglide/runtime';
import type { PageLoad } from './$types';

export const load: PageLoad = ({ params }) => {
	const locale = getLocale();
	const doc = getDoc(locale, params.slug);

	if (!doc) throw error(404, 'Página não encontrada / Page not found');

	return {
		Doc: doc.component,
		frontmatter: doc.frontmatter,
		raw: doc.raw,
		...getPrevNext(locale, doc.slug)
	};
};
