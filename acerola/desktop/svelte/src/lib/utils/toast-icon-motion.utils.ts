import gsap from 'gsap';
import type { Action } from 'svelte/action';
import { cn } from '$lib/utils/cn.utils';

export type ToastIconVariant = 'loading' | 'success' | 'error' | 'warning' | 'info';

const BURST_COLOR: Partial<Record<ToastIconVariant, string>> = {
	success: 'bg-green-500/35',
	error: 'bg-destructive/30'
};

// Cria e anima um anel que expande e desaparece atrás do ícone — dá peso ao resultado
// (sucesso/erro) sem depender só da troca de glifo. Remove o próprio elemento ao terminar.
function spawnBurst(node: HTMLElement, colorClass: string) {
	const ring = document.createElement('span');
	ring.className = cn('pointer-events-none absolute inset-0 rounded-full', colorClass);
	node.prepend(ring);
	gsap.fromTo(
		ring,
		{ scale: 0.3, opacity: 0.6 },
		{
			scale: 2.4,
			opacity: 0,
			duration: 0.55,
			ease: 'power2.out',
			onComplete: () => ring.remove()
		}
	);
}

/// Anima a entrada do ícone de um toast (`svelte-sonner`) sempre que o `type` muda —
/// `loading` -> `success`/`error`/etc monta um elemento novo no DOM (ver `Toast.svelte` da
/// lib), então este `use:` action roda uma vez por troca de estado, dando a sensação de
/// "morph" mesmo sendo ícones diferentes ocupando o mesmo slot.
export const toastIconEnter: Action<HTMLElement, ToastIconVariant> = (node, variant) => {
	node.classList.add('relative', 'inline-flex', 'items-center', 'justify-center');

	const ctx = gsap.context(() => {
		switch (variant) {
			case 'success':
				spawnBurst(node, BURST_COLOR.success!);
				gsap.fromTo(
					node,
					{ scale: 0.4, opacity: 0, rotate: -90 },
					{ scale: 1, opacity: 1, rotate: 0, duration: 0.4, ease: 'back.out(2.2)' }
				);
				break;
			case 'error':
				spawnBurst(node, BURST_COLOR.error!);
				gsap.fromTo(
					node,
					{ scale: 0.5, opacity: 0 },
					{
						scale: 1,
						opacity: 1,
						duration: 0.3,
						ease: 'power2.out',
						onComplete: () => {
							gsap.fromTo(
								node,
								{ x: -4 },
								{ x: 0, duration: 0.35, ease: 'elastic.out(1.4, 0.35)' }
							);
						}
					}
				);
				break;
			case 'warning':
			case 'info':
				gsap.fromTo(
					node,
					{ y: -6, opacity: 0 },
					{ y: 0, opacity: 1, duration: 0.28, ease: 'power2.out' }
				);
				break;
			default:
				gsap.fromTo(node, { opacity: 0 }, { opacity: 1, duration: 0.2 });
		}
	}, node);

	return {
		destroy() {
			ctx.revert();
		}
	};
};
