<script lang="ts" module>
	import CircleAlertIcon from '@lucide/svelte/icons/circle-alert';
	import InfoIcon from '@lucide/svelte/icons/info';
	import LightbulbIcon from '@lucide/svelte/icons/lightbulb';
	import OctagonAlertIcon from '@lucide/svelte/icons/octagon-alert';
	import type { Component } from 'svelte';

	export type AcerolaCalloutType = 'note' | 'tip' | 'caution' | 'danger';

	// Selo editorial (ícone + rótulo maiúsculo + régua fina à esquerda) em vez do
	// admonition padrão de biblioteca de UI (ícone + título numa caixa fechada) —
	// não é um Alert genérico nem um Card, é uma nota à margem do texto. Cada
	// variante define sua própria classe de acento em vez de tokens interpolados,
	// já que o Tailwind precisa da classe completa e literal pra gerar o CSS.
	const CALLOUT_CONFIG: Record<AcerolaCalloutType, { icon: Component; accent: string }> = {
		note: { icon: InfoIcon, accent: 'border-primary text-primary' },
		tip: { icon: LightbulbIcon, accent: 'border-chart-4 text-chart-4' },
		caution: { icon: CircleAlertIcon, accent: 'border-chart-2 text-chart-2' },
		danger: { icon: OctagonAlertIcon, accent: 'border-destructive text-destructive' }
	};
</script>

<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		type = 'note',
		title,
		children
	}: { type?: AcerolaCalloutType; title?: string; children: Snippet } = $props();

	const config = $derived(CALLOUT_CONFIG[type]);

	// `alert` é uma live region assertiva, reservada pra conteúdo urgente — usar
	// em avisos estáticos (note/tip/caution) faria o leitor de tela interromper o
	// usuário a cada navegação client-side só pra reler um aviso informativo.
	const role = $derived(type === 'danger' ? 'alert' : undefined);
</script>

<div
	{role}
	class={[
		'my-6 grid grid-cols-[auto_1fr] items-baseline gap-x-2.5 gap-y-1 border-l-2 py-0.5 pl-4',
		config.accent
	]}
>
	<config.icon size={14} class="translate-y-px" />
	<p class="m-0 text-xs font-semibold tracking-widest uppercase">{title ?? type}</p>
	<div class="col-start-2 text-sm text-foreground [&>p]:my-0">
		{@render children()}
	</div>
</div>
