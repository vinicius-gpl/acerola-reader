import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { scrollScale } from './scroll-scale';

describe('scrollScale action', () => {
	let element: HTMLDivElement;

	beforeEach(() => {
		element = document.createElement('div');
		const child1 = document.createElement('div');
		child1.className = 'child-card';
		const child2 = document.createElement('div');
		child2.className = 'child-card';
		element.appendChild(child1);
		element.appendChild(child2);
		document.body.appendChild(element);
	});

	afterEach(() => {
		element.remove();
		vi.clearAllMocks();
	});

	it('initializes scrub mode by default and returns destroy', () => {
		const action = scrollScale(element);
		expect(action).toBeDefined();
		expect(typeof action?.destroy).toBe('function');
		expect(() => action?.destroy?.()).not.toThrow();
	});

	it('initializes scale-up mode', () => {
		const action = scrollScale(element, { mode: 'scale-up', startScale: 0.9 });
		expect(action).toBeDefined();
		expect(typeof action?.destroy).toBe('function');
		expect(() => action?.destroy?.()).not.toThrow();
	});

	it('initializes stagger-grid mode with child selector', () => {
		const action = scrollScale(element, {
			mode: 'stagger-grid',
			childrenSelector: '.child-card',
			stagger: 0.1
		});
		expect(action).toBeDefined();
		expect(typeof action?.destroy).toBe('function');
		expect(() => action?.destroy?.()).not.toThrow();
	});

	it('initializes parallax-img mode', () => {
		const action = scrollScale(element, { mode: 'parallax-img' });
		expect(action).toBeDefined();
		expect(typeof action?.destroy).toBe('function');
		expect(() => action?.destroy?.()).not.toThrow();
	});

	it('exits early when prefers-reduced-motion is true', () => {
		const originalMatchMedia = window.matchMedia;
		window.matchMedia = vi.fn().mockImplementation((query) => ({
			matches: query === '(prefers-reduced-motion: reduce)',
			media: query
		}));

		const action = scrollScale(element);
		expect(action).toBeUndefined();

		window.matchMedia = originalMatchMedia;
	});
});
