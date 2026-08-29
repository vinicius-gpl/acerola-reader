import { render, screen, waitFor } from '@testing-library/svelte';
import { userEvent } from '@testing-library/user-event';
import { createRawSnippet } from 'svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import RootLayout from '../+layout.svelte';

const { mockGoto } = vi.hoisted(() => ({ mockGoto: vi.fn() }));

vi.mock('$app/navigation', () => ({ goto: mockGoto }));

vi.mock('$app/stores', async () => {
	const { writable: makeWritable } = await import('svelte/store');
	const store = makeWritable({ url: new URL('http://localhost/home') });
	return { page: store };
});

const { mockInvoke } = vi.hoisted(() => ({ mockInvoke: vi.fn() }));

vi.mock('@tauri-apps/api/core', () => ({
	invoke: (cmd: string, args: unknown) => mockInvoke(cmd, args),
	convertFileSrc: (path: string) => `asset://${path}`
}));

vi.mock('@tauri-apps/api/event', () => ({
	listen: () => Promise.resolve(vi.fn())
}));

const { mockMinimize, mockToggleMaximize, mockCloseWin } = vi.hoisted(() => ({
	mockMinimize: vi.fn(),
	mockToggleMaximize: vi.fn(),
	mockCloseWin: vi.fn()
}));

vi.mock('@tauri-apps/api/window', () => ({
	getCurrentWindow: () => ({
		theme: vi.fn().mockResolvedValue('light'),
		setTheme: vi.fn().mockResolvedValue(undefined),
		onThemeChanged: vi.fn().mockResolvedValue({ unlisten: vi.fn() }),
		minimize: mockMinimize,
		toggleMaximize: mockToggleMaximize,
		close: mockCloseWin
	})
}));

vi.mock('@tauri-apps/plugin-log', () => ({ error: vi.fn(), debug: vi.fn() }));

// checkStatus() do useOnboarding roda uma única vez, no import do módulo (top-level, não
// dentro do hook) — por isso o LazyStore precisa já devolver "completo" aqui na factory do
// vi.mock (hoisted antes de qualquer import), não em beforeEach: nesse ponto já é tarde.
vi.mock('@tauri-apps/plugin-store', () => ({
	load: vi.fn().mockResolvedValue({
		get: vi.fn().mockResolvedValue(undefined),
		set: vi.fn().mockResolvedValue(undefined),
		delete: vi.fn().mockResolvedValue(undefined),
		save: vi.fn().mockResolvedValue(undefined)
	}),
	LazyStore: class {
		constructor() {
			return {
				// só "onboarding_completed" precisa ser true — o mesmo LazyStore é
				// reaproveitado por use-theme.svelte.ts (chaves "theme"/"mode"), que quebra se
				// receber `true` como valor de tema/modo em vez de undefined (cai no default).
				get: vi.fn((key: string) =>
					Promise.resolve(key === 'onboarding_completed' ? true : undefined)
				),
				set: vi.fn().mockResolvedValue(undefined)
			};
		}
	}
}));

function setupInvokeMock(overrides: Record<string, unknown> = {}) {
	const defaults: Record<string, unknown> = {
		get_categories: [],
		get_all_comic_categories: [],
		get_package_family_name: 'No package identity'
	};
	mockInvoke.mockImplementation((cmd: string) =>
		Promise.resolve(cmd in overrides ? overrides[cmd] : (defaults[cmd] ?? undefined))
	);
}

function childrenSnippet() {
	return createRawSnippet(() => ({
		render: () => `<div data-testid="page-content">Conteúdo da página</div>`
	}));
}

describe('root +layout', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		setupInvokeMock();
	});

	it('renders the sidebar, header and page content once onboarding is complete', async () => {
		render(RootLayout, { props: { children: childrenSnippet() } });

		expect(await screen.findByTestId('page-content')).toBeInTheDocument();
		expect(screen.getByText('Acerola')).toBeInTheDocument();
	});

	it('calls the native window controls', async () => {
		const user = userEvent.setup();
		render(RootLayout, { props: { children: childrenSnippet() } });

		await screen.findByTestId('page-content');
		await waitFor(() => expect(mockInvoke).toHaveBeenCalled());

		await user.click(screen.getByRole('button', { name: /minimize|minimizar/i }));
		expect(mockMinimize).toHaveBeenCalled();

		await user.click(screen.getByRole('button', { name: /maximize|maximizar/i }));
		expect(mockToggleMaximize).toHaveBeenCalled();

		await user.click(screen.getByRole('button', { name: /close|fechar/i }));
		expect(mockCloseWin).toHaveBeenCalled();
	});

	it('opens the command palette on ctrl+k and navigates to a section', async () => {
		const user = userEvent.setup();
		render(RootLayout, { props: { children: childrenSnippet() } });

		await screen.findByTestId('page-content');

		await user.keyboard('{Control>}k{/Control}');

		const historyItem = await screen.findByText(/histórico|history/i);
		await user.click(historyItem);

		expect(mockGoto).toHaveBeenCalledWith('/history');
	});
});
