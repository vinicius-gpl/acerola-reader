<script module lang="ts">
	import { Button, type ButtonProps } from '$lib/components/ui/button';

	// Padrão "contorno": bg SÓLIDO colorido por padrão (não só no hover) com o ícone na cor
	// de foreground por cima — o bg contorna o ícone, não o contrário. `accent` é pra ações
	// sem significado semântico próprio (refresh, editar, navegar); as outras cores carregam
	// o significado da própria ação (perigo, sucesso, dispensar/cancelar).
	export type AcerolaButtonIconTone = 'accent' | 'destructive' | 'success' | 'muted';

	// hover:brightness-110 é o mesmo mecanismo usado no ícone do AcerolaHeroButton/
	// AcerolaAccordionCard/AcerolaToggleCard — funciona igual pra qualquer cor de bg, sem
	// precisar de uma variante /85 por tom.
	const TONE_CLASS: Record<AcerolaButtonIconTone, string> = {
		accent: 'bg-accent-hero text-accent-hero-foreground transition-[filter] hover:brightness-110',
		destructive:
			'bg-destructive text-destructive-foreground transition-[filter] hover:brightness-110',
		success: 'bg-chart-4 text-primary-foreground transition-[filter] hover:brightness-110',
		muted: 'bg-muted text-muted-foreground transition-[filter] hover:brightness-110'
	};

	export type AcerolaButtonIconProps = {
		events?: {
			onClick?: ButtonProps['onclick'];
		};
		ui?: Omit<ButtonProps, 'children' | 'class' | 'onclick'> & {
			class?: string;
			tone?: AcerolaButtonIconTone;
		};
	};
</script>

<script lang="ts">
	import { cn } from '$lib/utils/cn.utils';
	import type { Snippet } from 'svelte';

	let {
		children,
		events,
		ui
	}: AcerolaButtonIconProps & {
		children?: Snippet;
	} = $props();

	const uiProps = $derived.by(() => {
		const { class: _class, tone: _tone, ...rest } = ui ?? {};
		return rest;
	});
</script>

<Button
	class={cn(
		'size-10 rounded-xl p-0 enabled:cursor-pointer disabled:pointer-events-auto disabled:cursor-not-allowed',
		ui?.tone && TONE_CLASS[ui.tone],
		ui?.class
	)}
	onclick={events?.onClick}
	{...uiProps}
>
	{@render children?.()}
</Button>
