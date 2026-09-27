import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { card3D } from './card-3d';

describe('card3D action', () => {
	let element: HTMLDivElement;
	let originalMatchMedia: typeof window.matchMedia;

	beforeEach(() => {
		element = document.createElement('div');
		vi.spyOn(element, 'getBoundingClientRect').mockReturnValue({
			width: 200,
			height: 100,
			top: 10,
			left: 10,
			bottom: 110,
			right: 210,
			x: 10,
			y: 10,
			toJSON() {}
		});
		document.body.appendChild(element);

		originalMatchMedia = window.matchMedia;
		window.matchMedia = vi.fn().mockImplementation((query) => ({
			matches: query === '(hover: hover) and (pointer: fine)',
			media: query
		}));
	});

	afterEach(() => {
		element.remove();
		window.matchMedia = originalMatchMedia;
		vi.clearAllMocks();
	});

	it('adds card-3d and spotlight-card classes and returns destroy', () => {
		const action = card3D(element);
		expect(element).toHaveClass('card-3d');
		expect(element).toHaveClass('spotlight-card');
		expect(action).toBeDefined();
		expect(typeof action?.destroy).toBe('function');
		action?.destroy?.();
	});

	it('handles pointerenter, pointermove, and pointerleave events', () => {
		const action = card3D(element);

		element.dispatchEvent(new PointerEvent('pointerenter'));
		element.dispatchEvent(new PointerEvent('pointermove', { clientX: 50, clientY: 50 }));

		expect(element.style.getPropertyValue('--mouse-x')).toBeTruthy();
		expect(element.style.getPropertyValue('--mouse-y')).toBeTruthy();

		element.dispatchEvent(new PointerEvent('pointerleave'));

		action?.destroy?.();
	});

	it('disables spotlight when spotlight: false option is passed', () => {
		const action = card3D(element, { spotlight: false });
		expect(element).toHaveClass('card-3d');
		expect(element).not.toHaveClass('spotlight-card');
		action?.destroy?.();
	});

	it('exits early on coarse pointer devices', () => {
		window.matchMedia = vi.fn().mockImplementation(() => ({
			matches: false,
			media: ''
		}));

		const action = card3D(element);
		expect(action).toBeUndefined();
		expect(element).not.toHaveClass('card-3d');
	});

	it('exits early when prefers-reduced-motion is true', () => {
		window.matchMedia = vi.fn().mockImplementation((query) => ({
			matches: query === '(prefers-reduced-motion: reduce)',
			media: query
		}));

		const action = card3D(element);
		expect(action).toBeUndefined();
	});
});
