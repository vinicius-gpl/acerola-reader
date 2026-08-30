import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import LibraryTable from './library-table.svelte';

const libraries = [
	{ name: 'svelte', url: 'https://github.com/sveltejs/svelte', license: 'MIT' },
	{
		name: 'unrar',
		url: 'https://github.com/muja/unrar.rs',
		license: 'MIT OR Apache-2.0',
		note: '¹'
	}
];

describe('LibraryTable (mdsvex)', () => {
	it('renders one row per library, each as an external link to its repo', () => {
		render(LibraryTable, { props: { libraries } });

		const svelteLink = screen.getByRole('link', { name: 'svelte' });
		expect(svelteLink).toHaveAttribute('href', 'https://github.com/sveltejs/svelte');
		expect(svelteLink).toHaveAttribute('target', '_blank');
		expect(svelteLink).toHaveAttribute('rel', 'noopener noreferrer');

		expect(screen.getByRole('link', { name: 'unrar' })).toBeInTheDocument();
	});

	it('renders the license, appending the note when present', () => {
		render(LibraryTable, { props: { libraries } });

		expect(screen.getByText('MIT')).toBeInTheDocument();
		expect(
			screen.getByText((_, node) => node?.textContent === 'MIT OR Apache-2.0¹')
		).toBeInTheDocument();
	});

	it('uses the default English column labels when none are given', () => {
		render(LibraryTable, { props: { libraries } });

		expect(screen.getByRole('columnheader', { name: 'Library' })).toBeInTheDocument();
		expect(screen.getByRole('columnheader', { name: 'License' })).toBeInTheDocument();
	});

	it('uses custom column labels when provided', () => {
		render(LibraryTable, {
			props: { libraries, nameLabel: 'Biblioteca', licenseLabel: 'Licença' }
		});

		expect(screen.getByRole('columnheader', { name: 'Biblioteca' })).toBeInTheDocument();
		expect(screen.getByRole('columnheader', { name: 'Licença' })).toBeInTheDocument();
	});
});
