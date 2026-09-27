import type { Action } from 'svelte/action';
import { gsap } from 'gsap';

export interface Card3DOptions {
	maxRotation?: number;
	hoverScale?: number;
	spotlight?: boolean;
}

export const card3D: Action<HTMLElement, Card3DOptions | undefined> = (node, options = {}) => {
	if (typeof window === 'undefined') return;
	if (window.matchMedia?.('(prefers-reduced-motion: reduce)')?.matches) return;
	if (!window.matchMedia?.('(hover: hover) and (pointer: fine)')?.matches) return;

	const maxRotation = options?.maxRotation ?? 6;
	const hoverScale = options?.hoverScale ?? 1.02;
	const enableSpotlight = options?.spotlight ?? true;

	node.classList.add('card-3d');
	if (enableSpotlight) {
		node.classList.add('spotlight-card');
	}

	const setRotateX = gsap.quickTo(node, 'rotationX', { duration: 0.35, ease: 'power2.out' });
	const setRotateY = gsap.quickTo(node, 'rotationY', { duration: 0.35, ease: 'power2.out' });
	const setScaleX = gsap.quickTo(node, 'scaleX', { duration: 0.35, ease: 'power2.out' });
	const setScaleY = gsap.quickTo(node, 'scaleY', { duration: 0.35, ease: 'power2.out' });

	function setScale(val: number) {
		setScaleX(val);
		setScaleY(val);
	}

	let isInside = false;

	function onPointerEnter() {
		isInside = true;
		setScale(hoverScale);
	}

	function onPointerMove(e: PointerEvent) {
		if (!isInside) {
			isInside = true;
			setScale(hoverScale);
		}

		const rect = node.getBoundingClientRect();
		const x = e.clientX - rect.left;
		const y = e.clientY - rect.top;

		if (enableSpotlight) {
			node.style.setProperty('--mouse-x', `${x}px`);
			node.style.setProperty('--mouse-y', `${y}px`);
		}

		const percentX = (x / (rect.width || 1)) * 2 - 1; // -1 to 1
		const percentY = (y / (rect.height || 1)) * 2 - 1; // -1 to 1

		setRotateY(percentX * maxRotation);
		setRotateX(-percentY * maxRotation);
	}

	function onPointerLeave() {
		isInside = false;
		setRotateX(0);
		setRotateY(0);
		setScale(1);
	}

	node.addEventListener('pointerenter', onPointerEnter);
	node.addEventListener('pointermove', onPointerMove);
	node.addEventListener('pointerleave', onPointerLeave);

	return {
		destroy() {
			node.removeEventListener('pointerenter', onPointerEnter);
			node.removeEventListener('pointermove', onPointerMove);
			node.removeEventListener('pointerleave', onPointerLeave);
			gsap.killTweensOf(node);
		}
	};
};
