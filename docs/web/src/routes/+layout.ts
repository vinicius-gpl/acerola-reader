// The whole site is static content (no per-request server logic), so prerendering
// the whole thing gives Cloudflare a plain static deploy and gives Pagefind actual
// HTML files to index post-build.
export const prerender = true;
