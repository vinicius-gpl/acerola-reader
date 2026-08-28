import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import type { SidebarGroup } from '$lib/content/docs';
import MobileNav from './mobile-nav.svelte';

const groups: SidebarGroup[] = [
	{
		section: 'Primeiros passos',
		docs: [
			{
				locale: 'pt-br',
				slug: 'getting-started',
				component: {} as never,
				frontmatter: { title: 'Primeiros passos', section: 'Primeiros passos', order: 1 }
			}
		]
	}
];

describe('MobileNav', () => {
	it('renders the sidebar and nav controls when open', async () => {
		render(MobileNav, { props: { open: true, groups, activeSlug: 'getting-started' } });

		expect(await screen.findByRole('link', { name: 'Primeiros passos' })).toHaveAttribute(
			'href',
			'/docs/getting-started'
		);
		expect(screen.getByRole('button', { name: 'Mudar tema' })).toBeInTheDocument();
	});

	it('does not render sheet content when closed', () => {
		render(MobileNav, { props: { open: false, groups, activeSlug: 'getting-started' } });

		expect(screen.queryByRole('link', { name: 'Primeiros passos' })).not.toBeInTheDocument();
	});
});
