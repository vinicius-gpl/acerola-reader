import gsap from 'gsap';
import type { Action } from 'svelte/action';

/// Anima a entrada do selo de check dos cards de relay (e qualquer outro "badge de ativo")
/// sempre que ele monta — como fica dentro de um `{#if}`, cada vez que a fonte liga o nó é
/// recriado do zero, então a animação toca de novo a cada toggle sem precisar de `{#key}`.
/// Mesmo padrão de ação gsap pontual (sem `destroy`) de `toastIconEnter`.
export const checkBadgePop: Action<HTMLElement> = (node) => {
	gsap.fromTo(
		node,
		{ scale: 0, opacity: 0 },
		{ scale: 1, opacity: 1, duration: 0.3, ease: 'back.out(2.5)' }
	);
};
