import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import AcerolaNetworkConnectCard from './acerola-network-connect-card.svelte';
import { InvalidConnectionCodeError } from '$lib/utils/connection-code.utils';

vi.mock('svelte-sonner', () => ({
	toast: {
		loading: vi.fn(() => 'mock-toast-id'),
		success: vi.fn(),
		error: vi.fn()
	}
}));

describe('AcerolaNetworkConnectCard', () => {
	const mockOnConnect = vi.fn();

	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('disables the connect button while the input is empty', () => {
		render(AcerolaNetworkConnectCard, {
			props: {
				data: { connecting: false },
				events: { onConnect: mockOnConnect }
			}
		});

		expect(screen.getByRole('button', { name: /Connect|Conectar/i })).toBeDisabled();
	});

	it('calls onConnect with the pasted code and clears the input on success', async () => {
		mockOnConnect.mockResolvedValueOnce(undefined);

		render(AcerolaNetworkConnectCard, {
			props: {
				data: { connecting: false },
				events: { onConnect: mockOnConnect }
			}
		});

		const input = screen.getByPlaceholderText(/./i);
		await fireEvent.input(input, { target: { value: 'my-code' } });

		const button = screen.getByRole('button', { name: /Connect|Conectar/i });
		await fireEvent.click(button);

		expect(mockOnConnect).toHaveBeenCalledWith('my-code');
	});

	it('shows the invalid-code message when onConnect rejects with InvalidConnectionCodeError', async () => {
		mockOnConnect.mockRejectedValueOnce(new InvalidConnectionCodeError());

		render(AcerolaNetworkConnectCard, {
			props: {
				data: { connecting: false },
				events: { onConnect: mockOnConnect }
			}
		});

		const input = screen.getByPlaceholderText(/./i);
		await fireEvent.input(input, { target: { value: 'bad-code' } });
		await fireEvent.click(screen.getByRole('button', { name: /Connect|Conectar/i }));

		expect(await screen.findByText(/invalid code|código inválido/i)).toBeInTheDocument();
	});

	it('shows a generic error message when onConnect rejects with an unexpected error', async () => {
		mockOnConnect.mockRejectedValueOnce(new Error('boom'));

		render(AcerolaNetworkConnectCard, {
			props: {
				data: { connecting: false },
				events: { onConnect: mockOnConnect }
			}
		});

		const input = screen.getByPlaceholderText(/./i);
		await fireEvent.input(input, { target: { value: 'some-code' } });
		await fireEvent.click(screen.getByRole('button', { name: /Connect|Conectar/i }));

		expect(await screen.findByRole('button', { name: /Connect|Conectar/i })).toBeInTheDocument();
	});

	it('disables the connect button while connecting is true', () => {
		render(AcerolaNetworkConnectCard, {
			props: {
				data: { connecting: true },
				events: { onConnect: mockOnConnect }
			}
		});

		expect(screen.getByRole('button', { name: /Connect|Conectar/i })).toBeDisabled();
	});
});
