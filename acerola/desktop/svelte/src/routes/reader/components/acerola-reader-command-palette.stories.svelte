<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaReaderCommandPalette from './acerola-reader-command-palette.svelte';

	const { Story } = defineMeta({
		title: 'Páginas/Reader/AcerolaReaderCommandPalette',
		component: AcerolaReaderCommandPalette,
		tags: ['autodocs'],
		argTypes: {
			data: { control: 'object' },
			state: { control: 'object' },
			events: { control: 'object' }
		}
	});
</script>

<script lang="ts">
	// AcerolaReaderCommandPalette e totalmente controlado (state.open + events.onOpenChange) e
	// nao tem trigger proprio — cada story precisa do seu proprio estado local e de um botao
	// real pra abrir, senao a aba Docs mostra as variantes com a palette aberta ao mesmo tempo.
	let value = $state('');
	let openDefault = $state(false);
	let readingMode = $state<'vertical' | 'horizontal' | 'webtoon'>('vertical');
	let zoomMode = $state(false);

	const events = {
		onOpenChange: (nextOpen: boolean) => (openDefault = nextOpen),
		onValueChange: (nextValue: string) => (value = nextValue),
		onReadingModeChange: (nextMode: typeof readingMode) => (readingMode = nextMode),
		onToggleZoomMode: () => (zoomMode = !zoomMode),
		onZoomIn: () => {},
		onZoomOut: () => {},
		onResetZoom: () => {}
	};

	let zoomValue = $state('zoom');
	let openZoomMode = $state(false);
	const zoomEvents = {
		onOpenChange: (nextOpen: boolean) => (openZoomMode = nextOpen),
		onValueChange: (nextValue: string) => (zoomValue = nextValue),
		onReadingModeChange: () => {},
		onToggleZoomMode: () => {},
		onZoomIn: () => {},
		onZoomOut: () => {},
		onResetZoom: () => {}
	};
</script>

<Story name="Open" asChild>
	<div class="relative h-96 overflow-hidden rounded-lg border border-surface/40 bg-base">
		<button
			type="button"
			onclick={() => (openDefault = true)}
			class="m-4 rounded-lg border border-border bg-surface-elevated px-4 py-2 text-sm font-medium text-foreground hover:opacity-80"
		>
			Open Command Palette
		</button>
		<AcerolaReaderCommandPalette
			data={{ zoomMode }}
			state={{ open: openDefault, value, readingMode }}
			{events}
		/>
	</div>
</Story>

<Story name="Zoom Mode Active" asChild>
	<div class="relative h-96 overflow-hidden rounded-lg border border-surface/40 bg-base">
		<button
			type="button"
			onclick={() => (openZoomMode = true)}
			class="m-4 rounded-lg border border-border bg-surface-elevated px-4 py-2 text-sm font-medium text-foreground hover:opacity-80"
		>
			Open Command Palette
		</button>
		<AcerolaReaderCommandPalette
			data={{ zoomMode: true }}
			state={{ open: openZoomMode, value: zoomValue, readingMode: 'horizontal' }}
			events={zoomEvents}
		/>
	</div>
</Story>

<Story name="Closed" asChild>
	<div class="relative h-64 overflow-hidden rounded-lg border border-surface/40 bg-base">
		<AcerolaReaderCommandPalette
			data={{ zoomMode: false }}
			state={{ open: false, value: '', readingMode: 'vertical' }}
			{events}
		/>
	</div>
</Story>
