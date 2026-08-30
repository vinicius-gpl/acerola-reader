import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import AcerolaReaderModeToggle from './acerola-reader-mode-toggle.svelte';

function props(overrides = {}) {
	return {
		state: {
			value: 'vertical'
		},
		events: {
			onValueChange: vi.fn()
		},
		...overrides
	} as const;
}

describe('AcerolaReaderModeToggle', () => {
	it('renders all modes in default desktop variant', () => {
		const { container } = render(AcerolaReaderModeToggle, { props: props() });

		expect(screen.getByTitle('Paginado vertical')).toBeInTheDocument();
		expect(screen.getByTitle('Paginado horizontal')).toBeInTheDocument();
		expect(screen.getByTitle('Webtoon')).toBeInTheDocument();
		expect(container.querySelector('[role="group"]')?.className).toContain('md:flex');
	});

	it('renders mobile variant and custom class', () => {
		const { container } = render(AcerolaReaderModeToggle, {
			props: props({
				ui: {
					variant: 'mobile',
					class: 'reader-toggle-test'
				}
			})
		});

		const group = container.querySelector('[role="group"]');
		expect(group?.className).toContain('grid');
		expect(group?.className).toContain('md:hidden');
		expect(group?.className).toContain('reader-toggle-test');
	});

	it('marks active mode received from state', () => {
		render(AcerolaReaderModeToggle, {
			props: props({
				state: {
					value: 'webtoon'
				}
			})
		});

		expect(screen.getByTitle('Webtoon')).toHaveAttribute('data-state', 'on');
		expect(screen.getByTitle('Paginado vertical')).toHaveAttribute('data-state', 'off');
	});

	it('emits change for all valid modes', async () => {
		const user = userEvent.setup();
		const cases = [
			{ initial: 'horizontal', title: 'Paginado vertical', value: 'vertical' },
			{ initial: 'vertical', title: 'Paginado horizontal', value: 'horizontal' },
			{ initial: 'vertical', title: 'Webtoon', value: 'webtoon' }
		] as const;

		for (const entry of cases) {
			const toggleProps = props({
				state: {
					value: entry.initial
				}
			});
			const { unmount } = render(AcerolaReaderModeToggle, { props: toggleProps });

			await user.click(screen.getByTitle(entry.title));

			expect(toggleProps.events.onValueChange).toHaveBeenCalledWith(entry.value);
			unmount();
		}
	});
});
