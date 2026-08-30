import { render } from '@testing-library/svelte';
import { tick } from 'svelte';
import { describe, expect, it, vi } from 'vitest';
import { isReaderEditableTarget, useReaderZoom } from './use-reader-zoom.svelte';
import ReaderZoomHarness from '../../../../tests/harness/hooks/reader-zoom.svelte';

async function renderHook() {
	let zoom: ReturnType<typeof useReaderZoom> | undefined;

	render(ReaderZoomHarness, {
		props: {
			onReady: (hook) => {
				zoom = hook;
			}
		}
	});

	await tick();
	await Promise.resolve();

	return zoom!;
}

function createViewport(width = 400, height = 300) {
	const viewport = document.createElement('div');

	Object.defineProperty(viewport, 'scrollLeft', { value: 10, configurable: true });
	Object.defineProperty(viewport, 'scrollTop', { value: 20, configurable: true });
	viewport.getBoundingClientRect = vi.fn(
		() =>
			({
				left: 10,
				top: 20,
				width,
				height,
				right: width + 10,
				bottom: height + 20
			}) as DOMRect
	);

	return viewport;
}

describe('useReaderZoom', () => {
	it('starts with default zoom and no active zoom mode', async () => {
		const zoom = await renderHook();

		expect(zoom.zoomLevel).toBe(1);
		expect(zoom.zoomLabel).toBe('100%');
		expect(zoom.zoomMode).toBe(false);
		expect(zoom.isZoomed).toBe(false);
		expect(zoom.zoomLayerStyle).toContain('scale(1)');
	});

	it('applies quick zoom using pointer point as origin', async () => {
		const zoom = await renderHook();
		const viewport = createViewport();

		zoom.setViewport(viewport);
		zoom.toggleQuickZoom(new MouseEvent('click', { clientX: 110, clientY: 70 }));

		expect(zoom.zoomLevel).toBe(1.65);
		expect(zoom.isZoomed).toBe(true);
		expect(zoom.zoomLayerStyle).toContain('transform-origin: 110px 70px');
	});

	it('clamps zoom between minimum and maximum', async () => {
		const zoom = await renderHook();

		for (let index = 0; index < 30; index++) {
			zoom.zoomIn();
		}

		expect(zoom.zoomLevel).toBe(3);

		for (let index = 0; index < 30; index++) {
			zoom.zoomOut();
		}

		expect(zoom.zoomLevel).toBe(1);
		expect(zoom.isZoomed).toBe(false);
	});

	it('uses mouse wheel only when zoom mode is active', async () => {
		const zoom = await renderHook();
		const inactiveEvent = new WheelEvent('wheel', { deltaY: -100, cancelable: true });
		const inactivePrevent = vi.spyOn(inactiveEvent, 'preventDefault');

		zoom.handleWheel(inactiveEvent);

		expect(inactivePrevent).not.toHaveBeenCalled();
		expect(zoom.zoomLevel).toBe(1);

		const activeEvent = new WheelEvent('wheel', { deltaY: -100, cancelable: true });
		const activePrevent = vi.spyOn(activeEvent, 'preventDefault');

		zoom.toggleZoomMode();
		zoom.handleWheel(activeEvent);

		expect(activePrevent).toHaveBeenCalledOnce();
		expect(zoom.zoomLevel).toBe(1.15);
	});

	it('clamps offset during pan within viewport', async () => {
		const zoom = await renderHook();
		const viewport = createViewport(200, 100);

		zoom.setViewport(viewport);
		zoom.toggleQuickZoom();

		const pointerTarget = {
			setPointerCapture: vi.fn(),
			hasPointerCapture: vi.fn(() => true),
			releasePointerCapture: vi.fn()
		};

		zoom.handlePointerDown({
			button: 0,
			pointerId: 1,
			clientX: 0,
			clientY: 0,
			target: document.createElement('div'),
			currentTarget: pointerTarget,
			preventDefault: vi.fn()
		} as unknown as PointerEvent);

		zoom.handlePointerMove({
			clientX: 1000,
			clientY: 1000,
			preventDefault: vi.fn()
		} as unknown as PointerEvent);

		expect(zoom.isPanning).toBe(true);
		expect(zoom.zoomLayerStyle).toMatch(/translate3d\(64\.9.+px, 32\.4.+px, 0\)/);

		zoom.stopPan({
			pointerId: 1,
			currentTarget: pointerTarget
		} as unknown as PointerEvent);

		expect(zoom.isPanning).toBe(false);
		expect(pointerTarget.releasePointerCapture).toHaveBeenCalledWith(1);
	});

	it('detects editable targets of the reader', () => {
		const input = document.createElement('input');
		const textarea = document.createElement('textarea');
		const select = document.createElement('select');
		const editable = document.createElement('div');
		const plain = document.createElement('div');

		Object.defineProperty(editable, 'isContentEditable', { value: true });

		expect(isReaderEditableTarget(input)).toBe(true);
		expect(isReaderEditableTarget(textarea)).toBe(true);
		expect(isReaderEditableTarget(select)).toBe(true);
		expect(isReaderEditableTarget(editable)).toBe(true);
		expect(isReaderEditableTarget(plain)).toBe(false);
		expect(isReaderEditableTarget(null)).toBe(false);
	});
});
