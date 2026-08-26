import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import TopNav from './top-nav.svelte';

describe('TopNav', () => {
	it('renders the site name, docs link, and search trigger', () => {
		render(TopNav, { props: { onOpenSearch: vi.fn(), onOpenMobileNav: vi.fn() } });

		expect(screen.getByText('Acerola')).toBeInTheDocument();
		expect(screen.getByRole('link', { name: 'Docs' })).toHaveAttribute(
			'href',
			'/docs/getting-started'
		);
		expect(screen.getAllByText('Pesquisar na documentação...').length).toBeGreaterThan(0);
	});

	it('calls onOpenMobileNav when the menu button is clicked', async () => {
		const onOpenMobileNav = vi.fn();
		render(TopNav, { props: { onOpenSearch: vi.fn(), onOpenMobileNav } });
		const user = userEvent.setup();

		await user.click(screen.getByRole('button', { name: 'Alternar menu' }));

		expect(onOpenMobileNav).toHaveBeenCalledOnce();
	});

	it('calls onOpenSearch when a search trigger is clicked', async () => {
		const onOpenSearch = vi.fn();
		render(TopNav, { props: { onOpenSearch, onOpenMobileNav: vi.fn() } });
		const user = userEvent.setup();

		const [firstTrigger] = screen.getAllByRole('button', {
			name: /Pesquisar na documentação.../
		});
		await user.click(firstTrigger);

		expect(onOpenSearch).toHaveBeenCalledOnce();
	});
});
