import { m } from '$lib/paraglide/messages';

export type ReaderMode = 'vertical' | 'horizontal' | 'webtoon';

const MIN_ZOOM = 1;
const MAX_ZOOM = 3;
const QUICK_ZOOM = 1.65;
const ZOOM_STEP = 0.15;

type ZoomAnchor = MouseEvent | WheelEvent;

export function isReaderEditableTarget(target: EventTarget | null) {
	if (!(target instanceof HTMLElement)) return false;

	return (
		target.isContentEditable ||
		target.tagName === 'TEXTAREA' ||
		target.tagName === 'INPUT' ||
		target.tagName === 'SELECT'
	);
}

export function useReaderZoom() {
	let zoomLevel = $state(MIN_ZOOM);
	let zoomMode = $state(false);

	let panX = $state(0);
	let panY = $state(0);

	let zoomOriginX = $state(0);
	let zoomOriginY = $state(0);

	let isPanning = $state(false);
	let viewport = $state<HTMLElement | null>(null);
	let focusedImage = $state<HTMLImageElement | null>(null);

	let panStartX = 0;
	let panStartY = 0;
	let panOriginX = 0;
	let panOriginY = 0;

	const isZoomed = $derived(zoomLevel > MIN_ZOOM);

	const zoomPercent = $derived(Math.round(zoomLevel * 100));
	const zoomLabel = $derived(`${zoomPercent}%`);

	const zoomStatusLabel = $derived(
		zoomMode
			? m['pages.reader.zoom.scroll_status']({ zoom: zoomLabel })
			: m['pages.reader.zoom.status']({ zoom: zoomLabel })
	);

	const zoomLayerStyle = $derived(
		`transform: translate3d(${panX}px, ${panY}px, 0) scale(${zoomLevel}); transform-origin: ${zoomOriginX}px ${zoomOriginY}px;`
	);

	function setViewport(node: HTMLElement | null) {
		if (viewport === node) return;

		viewport = node;
		clampPan();
	}

	// A imagem em foco varia de tamanho/proporção por capítulo (1920x1080, 1720x1450, etc.) e é
	// exibida com `object-contain`, então o retângulo realmente ocupado por ela dentro do viewport
	// quase nunca é igual ao retângulo do viewport em si — usar o viewport puro como limite de pan
	// deixava sobrar área vazia fora da imagem em quadrinhos com proporção diferente da tela.
	function setFocusedImage(node: HTMLImageElement | null) {
		if (focusedImage === node) return;

		focusedImage = node;
		clampPan();
	}

	function resetPan() {
		panX = 0;
		panY = 0;
	}

	// Calcula o retângulo que a imagem realmente ocupa dentro do viewport (mesma lógica do
	// `object-contain` do CSS), usando o tamanho intrínseco da imagem — que independe de
	// transform/scale, ao contrário de `getBoundingClientRect()` num elemento já escalado.
	function fittedContentSize() {
		const rect = viewport?.getBoundingClientRect();
		const containerWidth = rect?.width ?? 0;
		const containerHeight = rect?.height ?? 0;

		const naturalWidth = focusedImage?.naturalWidth ?? 0;
		const naturalHeight = focusedImage?.naturalHeight ?? 0;

		if (!naturalWidth || !naturalHeight || !containerWidth || !containerHeight) {
			return { width: containerWidth, height: containerHeight };
		}

		const containerRatio = containerWidth / containerHeight;
		const naturalRatio = naturalWidth / naturalHeight;

		if (naturalRatio > containerRatio) {
			return { width: containerWidth, height: containerWidth / naturalRatio };
		}

		return { width: containerHeight * naturalRatio, height: containerHeight };
	}

	function panBounds() {
		const { width, height } = fittedContentSize();

		const extraX = Math.max(0, (width * (zoomLevel - MIN_ZOOM)) / 2);
		const extraY = Math.max(0, (height * (zoomLevel - MIN_ZOOM)) / 2);

		return {
			minX: -extraX,
			maxX: extraX,
			minY: -extraY,
			maxY: extraY
		};
	}

	function clampPan() {
		const bounds = panBounds();

		panX = Math.min(bounds.maxX, Math.max(bounds.minX, panX));
		panY = Math.min(bounds.maxY, Math.max(bounds.minY, panY));
	}

	function clampZoom(value: number) {
		return Math.round(Math.max(MIN_ZOOM, Math.min(value, MAX_ZOOM)) * 100) / 100;
	}

	function zoomAnchorPoint(anchor?: ZoomAnchor) {
		if (!viewport) {
			return {
				x: window.innerWidth / 2,
				y: window.innerHeight / 2
			};
		}

		const rect = viewport.getBoundingClientRect();

		if (anchor) {
			return {
				x: viewport.scrollLeft + anchor.clientX - rect.left,
				y: viewport.scrollTop + anchor.clientY - rect.top
			};
		}

		return {
			x: viewport.scrollLeft + rect.width / 2,
			y: viewport.scrollTop + rect.height / 2
		};
	}

	function setZoom(value: number, anchor?: ZoomAnchor) {
		const currentZoom = zoomLevel;
		const nextZoom = clampZoom(value);

		if (nextZoom === currentZoom) return;

		if (currentZoom === MIN_ZOOM && nextZoom > MIN_ZOOM) {
			const point = zoomAnchorPoint(anchor);

			zoomOriginX = point.x;
			zoomOriginY = point.y;
			resetPan();
		}

		if (nextZoom === MIN_ZOOM) {
			zoomLevel = nextZoom;
			resetPan();
			return;
		}

		zoomLevel = nextZoom;
		clampPan();
	}

	function zoomIn(anchor?: ZoomAnchor) {
		setZoom(zoomLevel + ZOOM_STEP, anchor);
	}

	function zoomOut(anchor?: ZoomAnchor) {
		setZoom(zoomLevel - ZOOM_STEP, anchor);
	}

	function resetZoom() {
		setZoom(MIN_ZOOM);
		zoomMode = false;
	}

	function forceResetZoom() {
		zoomLevel = MIN_ZOOM;
		zoomMode = false;
		resetPan();
		isPanning = false;
	}

	function toggleQuickZoom(anchor?: ZoomAnchor) {
		setZoom(isZoomed ? MIN_ZOOM : QUICK_ZOOM, anchor);
	}

	function toggleZoomMode() {
		if (zoomMode) {
			resetZoom();
			return;
		}

		zoomMode = true;
	}

	function handleWheel(event: WheelEvent) {
		if (isZoomed || zoomMode) {
			event.preventDefault();
		}

		if (!zoomMode) return;

		setZoom(zoomLevel + (event.deltaY < 0 ? ZOOM_STEP : -ZOOM_STEP), event);
	}

	function handlePointerDown(event: PointerEvent) {
		if (!isZoomed || event.button !== 0 || isReaderEditableTarget(event.target)) return;

		event.preventDefault();
		isPanning = true;
		panStartX = event.clientX;
		panStartY = event.clientY;
		panOriginX = panX;
		panOriginY = panY;
		(event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
	}

	function handlePointerMove(event: PointerEvent) {
		if (!isPanning) return;

		event.preventDefault();
		panX = panOriginX + event.clientX - panStartX;
		panY = panOriginY + event.clientY - panStartY;
		clampPan();
	}

	function stopPan(event: PointerEvent) {
		if (!isPanning) return;

		isPanning = false;

		const target = event.currentTarget as HTMLElement;
		if (target.hasPointerCapture(event.pointerId)) {
			target.releasePointerCapture(event.pointerId);
		}
	}

	return {
		setViewport,
		setFocusedImage,
		clampPan,
		resetPan,
		zoomIn,
		zoomOut,
		resetZoom,
		forceResetZoom,
		toggleQuickZoom,
		toggleZoomMode,
		handleWheel,
		handlePointerDown,
		handlePointerMove,
		stopPan,
		get isZoomed() {
			return isZoomed;
		},
		get zoomLevel() {
			return zoomLevel;
		},
		get zoomMode() {
			return zoomMode;
		},
		get isPanning() {
			return isPanning;
		},
		get zoomLabel() {
			return zoomLabel;
		},
		get zoomStatusLabel() {
			return zoomStatusLabel;
		},
		get zoomLayerStyle() {
			return zoomLayerStyle;
		}
	};
}

export type ReaderZoomController = ReturnType<typeof useReaderZoom>;
