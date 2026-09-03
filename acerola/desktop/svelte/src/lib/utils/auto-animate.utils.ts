import autoAnimate from '@formkit/auto-animate';
import type { AutoAnimateOptions, AutoAnimationPlugin } from '@formkit/auto-animate';
import type { Action } from 'svelte/action';

/// `autoAnimate()` retorna um `AnimationController` (com `enable`/`disable`/`isEnabled`), não o
/// `{destroy}` que o contrato de action do Svelte espera — este wrapper fecha essa diferença e
/// desliga a observação da lista quando o nó é destruído.
export const autoAnimateList: Action<
	HTMLElement,
	Partial<AutoAnimateOptions> | AutoAnimationPlugin | undefined
> = (node, options) => {
	const controller = autoAnimate(node, options);

	return {
		destroy() {
			controller.disable();
		}
	};
};
