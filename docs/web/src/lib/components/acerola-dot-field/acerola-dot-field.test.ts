import { render } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import AcerolaDotField from './acerola-dot-field.svelte';

describe('AcerolaDotField', () => {
	beforeEach(() => {
		globalThis.ResizeObserver = class {
			observe = vi.fn();
			unobserve = vi.fn();
			disconnect = vi.fn();
		} as unknown as typeof ResizeObserver;
	});

	it('renders canvas and svg container', () => {
		const { container } = render(AcerolaDotField);

		const canvas = container.querySelector('canvas');
		const svg = container.querySelector('svg');

		expect(canvas).toBeInTheDocument();
		expect(svg).toBeInTheDocument();
	});

	it('applies custom class names', () => {
		const { container } = render(AcerolaDotField, { props: { class: 'custom-dotfield' } });

		const div = container.firstElementChild;
		expect(div?.classList.contains('custom-dotfield')).toBe(true);
	});
});
