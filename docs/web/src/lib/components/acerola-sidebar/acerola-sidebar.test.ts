import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import type { SidebarGroup } from '$lib/content/docs';
import AcerolaSidebar from './acerola-sidebar.svelte';

// Doc titles are deliberately different from their section names below — the sidebar renders
// both a section heading and a doc link, and if their text matched, `getByText`/`getByRole`
// queries couldn't tell which one they found.
function makeGroups(): SidebarGroup[] {
	return [
		{
			section: 'Primeiros passos',
			docs: [
				{
					locale: 'pt-br',
					slug: 'getting-started',
					component: {} as never,
					raw: '',
					frontmatter: { title: 'Guia de instalação', section: 'Primeiros passos', order: 1 }
				}
			]
		},
		{
			section: 'Conceitos',
			docs: [
				{
					locale: 'pt-br',
					slug: 'architecture',
					component: {} as never,
					raw: '',
					frontmatter: { title: 'Visão geral da arquitetura', section: 'Conceitos', order: 1 }
				}
			]
		}
	];
}

describe('AcerolaSidebar', () => {
	it('renders every section and doc link', () => {
		render(AcerolaSidebar, { props: { groups: makeGroups(), activeSlug: 'architecture' } });

		expect(screen.getByText('Primeiros passos')).toBeInTheDocument();
		expect(screen.getByText('Conceitos')).toBeInTheDocument();
		expect(screen.getByRole('link', { name: 'Guia de instalação' })).toHaveAttribute(
			'href',
			'/docs/getting-started'
		);
		expect(screen.getByRole('link', { name: 'Visão geral da arquitetura' })).toHaveAttribute(
			'href',
			'/docs/architecture'
		);
	});

	it('highlights the link matching activeSlug', () => {
		render(AcerolaSidebar, { props: { groups: makeGroups(), activeSlug: 'architecture' } });

		const active = screen.getByRole('link', { name: 'Visão geral da arquitetura' });
		const inactive = screen.getByRole('link', { name: 'Guia de instalação' });

		expect(active.className).toContain('text-primary');
		expect(inactive.className).not.toContain('text-primary');
	});
});
