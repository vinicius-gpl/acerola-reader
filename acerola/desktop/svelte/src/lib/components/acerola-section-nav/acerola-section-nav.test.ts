import { render, screen } from '@testing-library/svelte';
import { userEvent } from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import AcerolaSectionNav from './acerola-section-nav.svelte';

const sections = [
	{ id: 'files', label: 'Arquivos' },
	{ id: 'library', label: 'Biblioteca' }
];

describe('AcerolaSectionNav', () => {
	it('renders one chip per section', () => {
		render(AcerolaSectionNav, {
			props: { data: { sections }, state: { activeId: 'files' }, events: { onSelect: vi.fn() } }
		});

		expect(screen.getByText('Arquivos')).toBeInTheDocument();
		expect(screen.getByText('Biblioteca')).toBeInTheDocument();
	});

	it('calls onSelect with the clicked section id', async () => {
		const onSelect = vi.fn();
		const user = userEvent.setup();
		render(AcerolaSectionNav, {
			props: { data: { sections }, state: { activeId: 'files' }, events: { onSelect } }
		});

		await user.click(screen.getByText('Biblioteca'));

		expect(onSelect).toHaveBeenCalledWith('library');
	});
});
