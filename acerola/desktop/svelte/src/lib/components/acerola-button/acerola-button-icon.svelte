<script module lang="ts">
	import { Button, type ButtonProps } from '$lib/components/ui/button';

	// Padrão "contorno": chip com bg colorido por padrão (não só no hover). `accent` é pra
	// ações sem significado semântico próprio (refresh, editar, navegar); as outras cores
	// carregam o significado da própria ação (perigo, sucesso, dispensar/cancelar).
	export type AcerolaButtonIconTone = 'accent' | 'destructive' | 'success' | 'muted';

	const TONE_CLASS: Record<AcerolaButtonIconTone, string> = {
		accent: 'bg-accent-hero/15 text-accent-hero hover:bg-accent-hero/25',
		destructive: 'bg-destructive/15 text-destructive hover:bg-destructive/25',
		success: 'bg-chart-4/15 text-chart-4 hover:bg-chart-4/25',
		muted: 'bg-muted text-muted-foreground hover:bg-muted/70'
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
