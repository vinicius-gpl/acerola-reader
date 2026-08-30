import { fireEvent, render, screen } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { describe, expect, it, vi } from 'vitest';
import type { ReaderMode, ReaderZoomController } from '../hooks/use-reader-zoom.svelte';
import AcerolaReaderViewport from './acerola-reader-viewport.svelte';

function children() {
	return createRawSnippet(() => ({
		render: () => '<div data-testid="reader-content">Reader content</div>'
	}));
}

function zoom(overrides: Partial<ReaderZoomController> = {}): ReaderZoomController {
	return {
		setViewport: vi.fn(),
		clampPan: vi.fn(),
		resetPan: vi.fn(),
		zoomIn: vi.fn(),
		zoomOut: vi.fn(),
		resetZoom: vi.fn(),
		forceResetZoom: vi.fn(),
		toggleQuickZoom: vi.fn(),
		toggleZoomMode: vi.fn(),
		handleWheel: vi.fn(),
		handlePointerDown: vi.fn(),
		handlePointerMove: vi.fn(),
		stopPan: vi.fn(),
		isZoomed: false,
		zoomLevel: 1,
		zoomMode: false,
		isPanning: false,
		zoomLabel: '100%',
		zoomStatusLabel: 'Zoom 100%' as ReaderZoomController['zoomStatusLabel'],
		zoomLayerStyle: 'transform: translate3d(0px, 0px, 0) scale(1);',
		...overrides
	};
}

function props(mode: ReaderMode = 'vertical', zoomController = zoom()) {
	return {
		data: {
			mode
		},
		context: {
			zoom: zoomController
		},
		children: children()
	} as const;
}

describe('AcerolaReaderViewport', () => {
	it('renders content and registers viewport on mount and destroy', () => {
		const zoomController = zoom();
		const { unmount } = render(AcerolaReaderViewport, { props: props('vertical', zoomController) });

		expect(screen.getByTestId('reader-content')).toHaveTextContent('Reader content');
		expect(zoomController.setViewport).toHaveBeenCalledWith(screen.getByRole('main'));

		unmount();
		expect(zoomController.setViewport).toHaveBeenCalledWith(null);
	});

	it('applies classes for reading modes without zoom', () => {
		const cases = [
			{ mode: 'horizontal', mainClass: 'overflow-x-auto', layerClass: 'w-max' },
			{ mode: 'vertical', mainClass: 'overflow-y-auto', layerClass: 'w-full' },
			{ mode: 'webtoon', mainClass: 'overflow-y-auto', layerClass: 'w-full' }
		] as const;

		for (const entry of cases) {
			const { container, unmount } = render(AcerolaReaderViewport, { props: props(entry.mode) });
			const main = screen.getByRole('main');
			const layer = container.querySelector('main > div');

			expect(main.className).toContain(entry.mainClass);
			expect(layer?.className).toContain(entry.layerClass);
			unmount();
		}
	});

	it('applies classes for zoom, pan and zoom mode', () => {
		const cases = [
			{ className: 'cursor-grab', zoom: zoom({ isZoomed: true }) },
			{ className: 'cursor-grabbing', zoom: zoom({ isZoomed: true, isPanning: true }) },
			{ className: 'cursor-zoom-in', zoom: zoom({ zoomMode: true }) }
		];

		for (const entry of cases) {
			const { unmount } = render(AcerolaReaderViewport, {
				props: props('vertical', entry.zoom)
			});

			expect(screen.getByRole('main').className).toContain(entry.className);
			unmount();
		}
	});

	it('applies zoom style on inner layer', () => {
		const zoomController = zoom({
			zoomLayerStyle: 'transform: translate3d(10px, 20px, 0) scale(1.5);'
		});
		const { container } = render(AcerolaReaderViewport, {
			props: props('vertical', zoomController)
		});

		expect(container.querySelector('main > div')).toHaveAttribute(
			'style',
			'transform: translate3d(10px, 20px, 0) scale(1.5);'
		);
	});

	it('forwards pointer, wheel and double click events to zoom', async () => {
		const zoomController = zoom();

		render(AcerolaReaderViewport, { props: props('vertical', zoomController) });
		const main = screen.getByRole('main');

		await fireEvent.wheel(main);
		await fireEvent.pointerDown(main);
		await fireEvent.pointerMove(main);
		await fireEvent.pointerUp(main);
		await fireEvent.pointerCancel(main);
		await fireEvent.lostPointerCapture(main);
		await fireEvent.dblClick(main);

		expect(zoomController.handleWheel).toHaveBeenCalledOnce();
		expect(zoomController.handlePointerDown).toHaveBeenCalledOnce();
		expect(zoomController.handlePointerMove).toHaveBeenCalledOnce();
		expect(zoomController.stopPan).toHaveBeenCalledTimes(3);
		expect(zoomController.toggleQuickZoom).toHaveBeenCalledOnce();
	});
});
