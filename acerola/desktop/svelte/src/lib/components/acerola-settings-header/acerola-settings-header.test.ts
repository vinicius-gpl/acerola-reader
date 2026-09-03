import { render, screen } from '@testing-library/svelte';
import { userEvent } from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import AcerolaSettingsHeader from './acerola-settings-header.svelte';

describe('AcerolaSettingsHeader', () => {
	it('renders the title', () => {
		render(AcerolaSettingsHeader, {
			props: { data: { title: 'Biblioteca' }, events: { onBack: vi.fn() } }
		});

		expect(screen.getByText('Biblioteca')).toBeInTheDocument();
	});

	it('calls onBack when the back button is clicked', async () => {
		const onBack = vi.fn();
		const user = userEvent.setup();
		render(AcerolaSettingsHeader, {
			props: { data: { title: 'Biblioteca' }, events: { onBack } }
		});

		await user.click(screen.getByRole('button'));

		expect(onBack).toHaveBeenCalledOnce();
	});
});
