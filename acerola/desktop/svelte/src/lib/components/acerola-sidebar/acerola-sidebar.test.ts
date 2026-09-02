import { render, screen } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import HistoryIcon from '@lucide/svelte/icons/history';
import HouseIcon from '@lucide/svelte/icons/house';
import AcerolaSidebar from './acerola-sidebar.svelte';

vi.mock('$app/state', () => ({
	page: { url: new URL('http://localhost/home') }
}));

const items = [
	{ href: '/home', label: 'Home', icon: HouseIcon },
	{ href: '/history', label: 'History', icon: HistoryIcon }
];

describe('AcerolaSidebar', () => {
	it('renders items always visible with the correct href', () => {
		render(AcerolaSidebar, { props: { data: { items } } });

		expect(screen.getByRole('link', { name: 'Home' })).toHaveAttribute('href', '/home');
		expect(screen.getByRole('link', { name: 'History' })).toHaveAttribute('href', '/history');
	});

	it('highlights the active item based on the current route', () => {
		render(AcerolaSidebar, { props: { data: { items } } });

		expect(screen.getByRole('link', { name: 'Home' }).className).toContain(
			'text-primary-foreground'
		);
		expect(screen.getByRole('link', { name: 'History' }).className).not.toContain(
			'text-primary-foreground'
		);
		expect(screen.getByRole('link', { name: 'Home' })).toHaveAttribute('aria-current', 'page');
		expect(screen.getByRole('link', { name: 'History' })).not.toHaveAttribute('aria-current');
	});

	it('renders navigation always visible, without collapsed state', () => {
		render(AcerolaSidebar, { props: { data: { items } } });

		expect(screen.getByRole('navigation')).toBeInTheDocument();
		expect(screen.getAllByRole('link')).toHaveLength(items.length);
	});
});
