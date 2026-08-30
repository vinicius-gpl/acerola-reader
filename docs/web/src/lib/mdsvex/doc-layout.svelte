<script lang="ts">
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import ChevronLeftIcon from '@lucide/svelte/icons/chevron-left';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import ChevronUpIcon from '@lucide/svelte/icons/chevron-up';
	import MinusIcon from '@lucide/svelte/icons/minus';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import RotateCcwIcon from '@lucide/svelte/icons/rotate-ccw';
	import { page } from '$app/state';
	import CopyMarkdownButton from '$lib/components/copy-markdown-button/copy-markdown-button.svelte';
	import { DOC_RAW_CONTEXT_KEY, type DocRawContext } from '$lib/content/doc-raw-context';
	import { useTheme } from '$lib/hooks/theme/use-theme.svelte';
	import { m } from '$lib/paraglide/messages';
	import { getContext, mount, type Snippet } from 'svelte';

	let {
		title,
		description,
		children
	}: { title?: string; description?: string; children: Snippet } = $props();

	const themeCtx = useTheme();
	const docRaw = getContext<DocRawContext | undefined>(DOC_RAW_CONTEXT_KEY);
	let container = $state<HTMLElement>();

	const MIN_SCALE = 0.5;
	const MAX_SCALE = 3;
	const PAN_STEP = 60;
	type Transform = { scale: number; x: number; y: number };
	const transforms = new WeakMap<HTMLElement, Transform>();

	function iconButton(labelText: string, icon: typeof MinusIcon, onClick: () => void) {
		const button = document.createElement('button');
		button.type = 'button';
		button.setAttribute('aria-label', labelText);
		button.className =
			'flex items-center justify-center rounded p-1 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground';
		mount(icon, { target: button, props: { size: 14 } });
		button.addEventListener('click', onClick);
		return button;
	}

	// Os diagramas são HTML puro vindo do pipeline de markdown (ver rehypeMermaid em
	// mdsvex.config.js), não templates Svelte — os controles precisam ser enxertados
	// em cada nó `.mermaid` de forma imperativa em vez de declarados no template.
	// Move/escala o diagrama no próprio lugar (como o visualizador de mermaid do
	// GitHub, que este espelha) em vez de abrir um dialog — um SVG clonado brigando
	// com o próprio layout de um dialog por espaço se mostrou bem mais frágil do que
	// transformar o SVG original.
	function addZoomControls(node: HTMLElement) {
		if (node.querySelector('[data-mermaid-controls]')) return;

		const svg = node.querySelector<SVGElement>('svg');
		if (!svg) return;

		transforms.set(node, { scale: 1, x: 0, y: 0 });

		const zoomLabel = document.createElement('span');
		zoomLabel.className = 'text-center text-xs tabular-nums text-muted-foreground';
		zoomLabel.textContent = '100%';

		function apply() {
			const t = transforms.get(node)!;
			svg!.style.transform = `translate(${t.x}px, ${t.y}px) scale(${t.scale})`;
			svg!.style.transformOrigin = 'center';
			zoomLabel.textContent = Math.round(t.scale * 100) + '%';
		}

		function pan(dx: number, dy: number) {
			const t = transforms.get(node)!;
			t.x += dx;
			t.y += dy;
			apply();
		}

		function zoomBy(delta: number) {
			const t = transforms.get(node)!;
			t.scale = Math.min(MAX_SCALE, Math.max(MIN_SCALE, t.scale + delta));
			apply();
		}

		function reset() {
			transforms.set(node, { scale: 1, x: 0, y: 0 });
			apply();
		}

		const pad = document.createElement('div');
		pad.className = 'grid grid-cols-3 grid-rows-3 place-items-center';
		const cell = (col: number, row: number, el: HTMLElement) => {
			el.style.gridColumn = String(col);
			el.style.gridRow = String(row);
			return el;
		};
		pad.append(
			cell(
				2,
				1,
				iconButton(m['mermaid.pan_up'](), ChevronUpIcon, () => pan(0, PAN_STEP))
			),
			cell(
				1,
				2,
				iconButton(m['mermaid.pan_left'](), ChevronLeftIcon, () => pan(PAN_STEP, 0))
			),
			cell(2, 2, iconButton(m['mermaid.reset'](), RotateCcwIcon, reset)),
			cell(
				3,
				2,
				iconButton(m['mermaid.pan_right'](), ChevronRightIcon, () => pan(-PAN_STEP, 0))
			),
			cell(
				2,
				3,
				iconButton(m['mermaid.pan_down'](), ChevronDownIcon, () => pan(0, -PAN_STEP))
			)
		);

		const zoomStack = document.createElement('div');
		zoomStack.className = 'flex flex-col items-center gap-0.5';
		zoomStack.append(
			iconButton(m['mermaid.zoom_in'](), PlusIcon, () => zoomBy(0.25)),
			zoomLabel,
			iconButton(m['mermaid.zoom_out'](), MinusIcon, () => zoomBy(-0.25))
		);

		const toolbar = document.createElement('div');
		toolbar.dataset.mermaidControls = '';
		toolbar.className =
			'absolute bottom-2 right-2 z-10 flex items-center gap-1.5 rounded-md border border-border bg-background/90 p-1.5 opacity-0 backdrop-blur transition-opacity';
		toolbar.append(pad, zoomStack);
		node.appendChild(toolbar);
	}

	// mermaid.run() substitui o conteúdo de cada nó por um <svg> renderizado, então
	// o fonte original do diagrama é capturado aqui na primeira passagem e reusado
	// para resetar o nó antes de cada nova renderização (troca de paleta/modo).
	const sources = new Map<HTMLElement, string>();

	async function renderMermaid() {
		const nodes = container?.querySelectorAll<HTMLElement>('.mermaid');
		if (!nodes || nodes.length === 0) return;

		const { default: mermaid } = await import('mermaid');
		const styles = getComputedStyle(document.documentElement);
		const primaryColor = styles.getPropertyValue('--primary').trim();
		const lineColor = styles.getPropertyValue('--border').trim();
		const textColor = styles.getPropertyValue('--foreground').trim();
		const background = styles.getPropertyValue('--card').trim();

		mermaid.initialize({
			startOnLoad: false,
			theme: themeCtx.resolved === 'dark' ? 'dark' : 'default',
			themeVariables: {
				primaryColor: background,
				primaryTextColor: textColor,
				primaryBorderColor: primaryColor,
				lineColor,
				textColor,
				background
			}
		});

		for (const node of nodes) {
			if (!sources.has(node)) sources.set(node, node.textContent ?? '');
			node.textContent = sources.get(node) ?? '';
			node.removeAttribute('data-processed');
		}

		await mermaid.run({ nodes: [...nodes] });

		for (const node of nodes) addZoomControls(node);
	}

	$effect(() => {
		// Re-render whenever the active palette or light/dark mode changes.
		themeCtx.resolved;
		themeCtx.theme;
		renderMermaid();
	});
</script>

<article bind:this={container} class="doc-content max-w-none">
	{#if title}
		<div class="mb-2 flex items-center gap-2">
			<h1 class="font-heading text-4xl font-semibold">{title}</h1>
			{#if docRaw}
				<CopyMarkdownButton
					raw={docRaw.value}
					url={page.url.href}
					label={m['doc.copy_markdown']()}
					copiedLabel={m['doc.copy_markdown_copied']()}
				/>
			{/if}
		</div>
	{/if}
	{#if description}
		<p class="mb-8 text-lg text-muted-foreground">{description}</p>
	{/if}
	{@render children()}
</article>
