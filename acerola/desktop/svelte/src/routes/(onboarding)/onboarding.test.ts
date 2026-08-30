import { render, screen } from '@testing-library/svelte';
import { userEvent } from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const mockStoreMethods = vi.hoisted(() => ({
	get: vi.fn(() => Promise.resolve(null)),
	set: vi.fn(() => Promise.resolve()),
	save: vi.fn(() => Promise.resolve())
}));

vi.mock('@tauri-apps/plugin-store', () => ({
	load: vi.fn(() => mockStoreMethods),
	LazyStore: vi.fn().mockImplementation(function () {
		return mockStoreMethods;
	})
}));

vi.mock('$app/navigation', () => ({
	goto: vi.fn()
}));

vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn()
}));

import Onboarding from './acerola-onboarding.svelte';
import { useOnboarding } from '$lib/hooks/onboarding/use-onboarding.svelte';

describe('Onboarding Component', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		useOnboarding().setStep(0);
	});

	it('renders the first step (Welcome) by default', () => {
		render(Onboarding);

		expect(screen.getByText('Acerola')).toBeInTheDocument();
		expect(screen.getByText('Bem-vindo ao Acerola, seu leitor de quadrinhos.')).toBeInTheDocument();
		expect(screen.getByText('Começar')).toBeInTheDocument();
	});

	it('advances to Language step when clicking Start', async () => {
		const user = userEvent.setup();
		render(Onboarding);

		await user.click(screen.getByText('Começar'));

		expect(screen.getByText('Selecionar Idioma')).toBeInTheDocument();
		expect(screen.getByText('Escolha o idioma preferido para a aplicação.')).toBeInTheDocument();
	});

	it('advances through Language steps to Formats when clicking Next', async () => {
		const user = userEvent.setup();
		render(Onboarding);

		await user.click(screen.getByText('Começar'));
		await user.click(screen.getByText('Próximo'));

		expect(screen.getByText('Formatos Suportados')).toBeInTheDocument();
		expect(screen.getByText('CBZ')).toBeInTheDocument();
		expect(screen.getByText('CBR')).toBeInTheDocument();
		expect(screen.getByText('PDF')).toBeInTheDocument();
	});
});
