import gsap from 'gsap';
import type { Action } from 'svelte/action';
import { cn } from '$lib/utils/cn.utils';

type TrackedProp = 'x' | 'y' | 'width' | 'height';

export type SlidingIndicatorOptions = {
	/// Seletor (relativo ao container) do item atualmente ativo — lido via atributo, não via
	/// valor reativo (`aria-current="page"` na sidebar, `data-state="on"` no toggle group), pra
	/// action ficar pura DOM e não precisar de um `update()` amarrado a uma rune.
	selector: string;
	/// Classes do elemento indicador injetado — controla a aparência (pill atrás de um ícone,
	/// barra fina embaixo de uma aba). Propriedades de posição/tamanho que a classe já fixa via
	/// CSS (ex.: `bottom-0 h-1` numa barra) devem ficar de fora de `track` abaixo.
	indicatorClass: string;
	duration?: number;
	/// Quais propriedades da caixa do item ativo espelhar no indicador — o resto fica fixo pelas
	/// classes CSS acima. Default: todas (caso "pill" que cobre a caixa inteira).
	track?: TrackedProp[];
};

const DEFAULT_TRACK: TrackedProp[] = ['x', 'y', 'width', 'height'];
const DEFAULT_DURATION = 0.22;

/// Substitui uma troca instantânea de classe (ex.: `bg-primary` pulando de item pra item) por um
/// indicador que desliza até a posição/tamanho do item ativo — reaplicado sempre que o atributo
/// que marca "ativo" muda no container (via `MutationObserver`, não via prop reativa).
export const slidingIndicator: Action<HTMLElement, SlidingIndicatorOptions> = (node, opts) => {
	node.classList.add('relative');

	const indicator = document.createElement('div');
	indicator.className = cn(
		'pointer-events-none absolute top-0 left-0 opacity-0',
		opts.indicatorClass
	);
	node.prepend(indicator);

	const track = opts.track ?? DEFAULT_TRACK;
	let tween: gsap.core.Tween | undefined;

	function snap(animate: boolean) {
		const active = node.querySelector<HTMLElement>(opts.selector);
		tween?.kill();

		if (!active) {
			gsap.set(indicator, { opacity: 0 });
			return;
		}

		const containerRect = node.getBoundingClientRect();
		const rect = active.getBoundingClientRect();
		const full: Record<TrackedProp, number> = {
			x: rect.left - containerRect.left,
			y: rect.top - containerRect.top,
			width: rect.width,
			height: rect.height
		};

		const target: Record<string, number> = { opacity: 1 };
		for (const key of track) target[key] = full[key];

		if (animate) {
			tween = gsap.to(indicator, {
				...target,
				duration: opts.duration ?? DEFAULT_DURATION,
				ease: 'power2.out'
			});
		} else {
			gsap.set(indicator, target);
		}
	}

	snap(false);
	// Métricas de fonte só assentam depois do primeiro paint — sem isso, um item com texto
	// pode medir a largura errada até o layout se estabilizar de vez.
	document.fonts?.ready?.then(() => snap(false));

	const mutationObserver = new MutationObserver(() => snap(true));
	mutationObserver.observe(node, {
		attributes: true,
		subtree: true,
		attributeFilter: ['aria-current', 'data-state']
	});

	const resizeObserver = new ResizeObserver(() => snap(false));
	resizeObserver.observe(node);

	return {
		destroy() {
			mutationObserver.disconnect();
			resizeObserver.disconnect();
			tween?.kill();
			indicator.remove();
		}
	};
};
