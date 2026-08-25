import { fileURLToPath } from 'url';
import rehypeAutolinkHeadings from 'rehype-autolink-headings';
import rehypePrettyCode from 'rehype-pretty-code';
import rehypeSlug from 'rehype-slug';
import remarkGfm from 'remark-gfm';

// mdsvex resolves a relative `layout` path against each individual markdown file's
// directory, not the project root — an absolute path is needed since content lives
// several levels deep under src/content/docs/{locale}/.
const docLayout = fileURLToPath(new URL('./src/lib/mdsvex/doc-layout.svelte', import.meta.url));

/** @type {import('mdsvex').MdsvexOptions} */
const config = {
	extensions: ['.md'],
	layout: docLayout,
	remarkPlugins: [remarkGfm],
	rehypePlugins: [
		rehypeSlug,
		[rehypeAutolinkHeadings, { behavior: 'wrap', properties: { class: 'heading-anchor' } }],
		[
			rehypePrettyCode,
			{ theme: { light: 'github-light', dark: 'github-dark' }, keepBackground: false }
		]
	]
};

export default config;
