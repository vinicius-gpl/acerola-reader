import type { Action } from 'svelte/action';
import { gsap } from 'gsap';
import { ScrollTrigger } from 'gsap/ScrollTrigger';

let registered = false;

function ensureScrollTrigger() {
	if (!registered && typeof window !== 'undefined') {
		gsap.registerPlugin(ScrollTrigger);
		registered = true;
	}
}

export interface ScrollRevealOptions {
	y?: number;
	duration?: number;
	delay?: number;
	ease?: string;
	start?: string;
	once?: boolean;
}

export const scrollReveal: Action<HTMLElement, ScrollRevealOptions | undefined> = (
	node,
	options = {}
) => {
	if (typeof window === 'undefined') return;
	if (window.matchMedia?.('(prefers-reduced-motion: reduce)')?.matches) return;

	ensureScrollTrigger();

	const y = options?.y ?? 24;
	const duration = options?.duration ?? 0.6;
	const delay = options?.delay ?? 0;
	const ease = options?.ease ?? 'power2.out';
	const start = options?.start ?? 'top 85%';
	const once = options?.once ?? true;

	const tween = gsap.fromTo(
		node,
		{ opacity: 0, y },
		{
			opacity: 1,
			y: 0,
			duration,
			delay,
			ease,
			scrollTrigger: {
				trigger: node,
				start,
				once
			}
		}
	);

	return {
		destroy() {
			tween.scrollTrigger?.kill();
			tween.kill();
		}
	};
};
