import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { scrollReveal } from './scroll-reveal';

describe('scrollReveal action', () => {
	let element: HTMLDivElement;

	beforeEach(() => {
		element = document.createElement('div');
		document.body.appendChild(element);
	});

	afterEach(() => {
		element.remove();
		vi.clearAllMocks();
	});

	it('initializes and returns a destroy function', () => {
		const action = scrollReveal(element);
		expect(action).toBeDefined();
		expect(typeof action?.destroy).toBe('function');
		expect(() => action?.destroy?.()).not.toThrow();
	});

	it('accepts custom options without crashing', () => {
		const action = scrollReveal(element, { y: 40, duration: 1, delay: 0.2, once: false });
		expect(action).toBeDefined();
		expect(() => action?.destroy?.()).not.toThrow();
	});

	it('exits early when prefers-reduced-motion is true', () => {
		const originalMatchMedia = window.matchMedia;
		window.matchMedia = vi.fn().mockImplementation((query) => ({
			matches: query === '(prefers-reduced-motion: reduce)',
			media: query
		}));

		const action = scrollReveal(element);
		expect(action).toBeUndefined();

		window.matchMedia = originalMatchMedia;
	});
});
