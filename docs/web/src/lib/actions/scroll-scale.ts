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

export interface ScrollScaleOptions {
	/**
	 * Animation mode:
	 * - 'scrub': continuous scroll-linked scale (scales up as it enters viewport, subtly scales down as it exits)
	 * - 'scale-up': one-off smooth scale entrance when entering viewport
	 * - 'stagger-grid': reveals child elements with a staggered scale pop
	 * - 'parallax-img': subtle continuous scale & parallax for preview images
	 */
	mode?: 'scrub' | 'scale-up' | 'stagger-grid' | 'parallax-img';
	startScale?: number;
	endScale?: number;
	exitScale?: number;
	start?: string;
	end?: string;
	duration?: number;
	ease?: string;
	scrub?: boolean | number;
	stagger?: number;
	childrenSelector?: string;
	once?: boolean;
}

export const scrollScale: Action<HTMLElement, ScrollScaleOptions | undefined> = (
	node,
	options = {}
) => {
	if (typeof window === 'undefined') return;
	if (window.matchMedia?.('(prefers-reduced-motion: reduce)')?.matches) return;

	ensureScrollTrigger();

	const mode = options?.mode ?? 'scrub';
	let animation: gsap.core.Tween | gsap.core.Timeline | null = null;
	const triggers: ScrollTrigger[] = [];

	if (mode === 'scrub') {
		const startScale = options.startScale ?? 0.88;
		const endScale = options.endScale ?? 1.0;
		const exitScale = options.exitScale ?? 0.96;
		const scrubVal = typeof options.scrub === 'number' ? options.scrub : (options.scrub ?? 0.8);

		const tl = gsap.timeline({
			scrollTrigger: {
				trigger: node,
				start: options.start ?? 'top 95%',
				end: options.end ?? 'bottom 15%',
				scrub: scrubVal
			}
		});

		tl.fromTo(
			node,
			{ scale: startScale, opacity: 0.5, y: 35 },
			{ scale: endScale, opacity: 1, y: 0, ease: 'power2.out', duration: 0.6 }
		);

		if (exitScale !== endScale) {
			tl.to(node, {
				scale: exitScale,
				opacity: 0.88,
				ease: 'power1.in',
				duration: 0.4
			});
		}

		animation = tl;
		if (tl.scrollTrigger) triggers.push(tl.scrollTrigger);
	} else if (mode === 'scale-up') {
		const startScale = options.startScale ?? 0.88;
		const endScale = options.endScale ?? 1.0;

		const tween = gsap.fromTo(
			node,
			{ scale: startScale, opacity: 0, y: 28 },
			{
				scale: endScale,
				opacity: 1,
				y: 0,
				duration: options.duration ?? 0.75,
				ease: options.ease ?? 'power3.out',
				scrollTrigger: {
					trigger: node,
					start: options.start ?? 'top 85%',
					once: options.once ?? true
				}
			}
		);

		animation = tween;
		if (tween.scrollTrigger) triggers.push(tween.scrollTrigger);
	} else if (mode === 'stagger-grid') {
		const selector = options.childrenSelector;
		const items = selector ? node.querySelectorAll(selector) : node.children;

		if (items.length > 0) {
			const tween = gsap.fromTo(
				items,
				{
					scale: options.startScale ?? 0.8,
					opacity: 0,
					y: 30
				},
				{
					scale: options.endScale ?? 1.0,
					opacity: 1,
					y: 0,
					duration: options.duration ?? 0.65,
					stagger: options.stagger ?? 0.07,
					ease: options.ease ?? 'back.out(1.4)',
					scrollTrigger: {
						trigger: node,
						start: options.start ?? 'top 85%',
						once: options.once ?? true
					}
				}
			);

			animation = tween;
			if (tween.scrollTrigger) triggers.push(tween.scrollTrigger);
		}
	} else if (mode === 'parallax-img') {
		const selector = options.childrenSelector;
		const target = selector ? node.querySelector(selector) : node;

		if (target) {
			const tween = gsap.fromTo(
				target,
				{ scale: options.startScale ?? 0.94, y: 15 },
				{
					scale: options.endScale ?? 1.06,
					y: -15,
					ease: 'none',
					scrollTrigger: {
						trigger: node,
						start: options.start ?? 'top bottom',
						end: options.end ?? 'bottom top',
						scrub: typeof options.scrub === 'number' ? options.scrub : (options.scrub ?? 1)
					}
				}
			);

			animation = tween;
			if (tween.scrollTrigger) triggers.push(tween.scrollTrigger);
		}
	}

	return {
		destroy() {
			triggers.forEach((st) => st.kill());
			animation?.kill();
		}
	};
};
