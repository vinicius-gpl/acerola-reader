import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { GITHUB_URL } from '$lib/constants/site';
import NavControls from './nav-controls.svelte';

// setLocale() ultimately calls window.location.href = ... / window.location.reload(), which
// jsdom doesn't implement — mock it so the locale-toggle test doesn't trigger real navigation.
vi.mock('$lib/paraglide/runtime', async (importOriginal) => {
	const actual = await importOriginal<typeof import('$lib/paraglide/runtime')>();
	return { ...actual, setLocale: vi.fn() };
});

const { setLocale } = await import('$lib/paraglide/runtime');

beforeEach(() => {
	vi.mocked(setLocale).mockClear();
});

describe('NavControls', () => {
	it('renders the current locale, theme controls, and a GitHub link', () => {
		render(NavControls);

		expect(screen.getByRole('button', { name: 'PT-BR' })).toBeInTheDocument();
		expect(screen.getByRole('button', { name: 'Mudar tema' })).toBeInTheDocument();
		expect(screen.getByRole('button', { name: 'Mudar paleta de cores' })).toBeInTheDocument();
		expect(screen.getByRole('link', { name: 'GitHub' })).toHaveAttribute('href', GITHUB_URL);
	});

	it('toggles to the next locale on click', async () => {
		render(NavControls);
		const user = userEvent.setup();

		await user.click(screen.getByRole('button', { name: 'PT-BR' }));

		expect(setLocale).toHaveBeenCalledWith('en');
	});
});
