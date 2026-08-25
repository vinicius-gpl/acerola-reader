import { fileURLToPath } from 'url';
import rehypeAutolinkHeadings from 'rehype-autolink-headings';
import rehypePrettyCode from 'rehype-pretty-code';
import rehypeSlug from 'rehype-slug';
import remarkGfm from 'remark-gfm';
import { visit } from 'unist-util-visit';

// mdsvex resolves a relative `layout` path against each individual markdown file's
// directory, not the project root — an absolute path is needed since content lives
// several levels deep under src/content/docs/{locale}/.
const docLayout = fileURLToPath(new URL('./src/lib/mdsvex/doc-layout.svelte', import.meta.url));

// Mermaid renders client-side (see doc-layout.svelte), so ```mermaid fences must be
// pulled out before rehypePrettyCode/Shiki gets a chance to syntax-highlight them as
// a regular code block. The source is kept as the div's text content (not a data
// attribute — the diagram source freely contains quotes/newlines that don't survive
// round-tripping through an HTML attribute) so the client can read it back before
// mermaid.run() replaces it with an <svg> on the first render.
function rehypeMermaid() {
	return (tree) => {
		visit(tree, 'element', (node, index, parent) => {
			if (node.tagName !== 'pre' || !parent || index === null) return;

			const code = node.children.find(
				(child) => child.type === 'element' && child.tagName === 'code'
			);
			const className = code?.properties?.className;
			if (!Array.isArray(className) || !className.includes('language-mermaid')) return;

			parent.children[index] = {
				type: 'element',
				tagName: 'div',
				properties: { className: ['mermaid', 'not-prose'] },
				children: code.children
			};
		});
	};
}

/** @type {import('mdsvex').MdsvexOptions} */
const config = {
	extensions: ['.md'],
	layout: docLayout,
	// mdsvex's built-in Prism highlighter runs before the rehype stage and would
	// otherwise swallow every code fence as raw HTML before rehypeMermaid/rehypePrettyCode
	// below ever see it — disable it so Shiki (via rehypePrettyCode) does the highlighting.
	highlight: false,
	remarkPlugins: [remarkGfm],
	rehypePlugins: [
		rehypeSlug,
		[rehypeAutolinkHeadings, { behavior: 'wrap', properties: { class: 'heading-anchor' } }],
		rehypeMermaid,
		[
			rehypePrettyCode,
			{ theme: { light: 'github-light', dark: 'github-dark' }, keepBackground: false }
		]
	]
};

export default config;
