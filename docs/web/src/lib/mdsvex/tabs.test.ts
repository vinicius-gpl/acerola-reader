import { render, screen } from '@testing-library/svelte';
import { userEvent } from '@testing-library/user-event';
import { createRawSnippet } from 'svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import Tabs from './tabs.svelte';

function snippet(text: string) {
	return createRawSnippet(() => ({
		render: () => `<p>${text}</p>`
	}));
}

describe('Tabs (mdsvex)', () => {
	beforeEach(() => {
		// bits-ui anima a troca de aba — jsdom expõe Element.prototype.animate mas devolve
		// undefined em vez de um Animation, quebrando o componente ao trocar de valor.
		Element.prototype.animate = vi.fn().mockImplementation(() => ({
			finished: Promise.resolve(),
			cancel: vi.fn(),
			finish: vi.fn(),
			pause: vi.fn(),
			play: vi.fn(),
			reverse: vi.fn(),
			onfinish: null,
			oncancel: null
		})) as unknown as typeof Element.prototype.animate;
	});

	it('shows the first tab content by default and switches on click', async () => {
		const user = userEvent.setup();
		render(Tabs, {
			props: {
				items: [
					{ value: 'npm', label: 'npm', content: snippet('npm install acerola') },
					{ value: 'yarn', label: 'yarn', content: snippet('yarn add acerola') }
				]
			}
		});

		expect(screen.getByText('npm install acerola')).toBeVisible();

		await user.click(screen.getByRole('tab', { name: 'yarn' }));

		expect(await screen.findByText('yarn add acerola')).toBeVisible();
	});
});
