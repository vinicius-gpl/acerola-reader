<script module lang="ts">
	import type { ToggleGroup as ToggleGroupPrimitive } from 'bits-ui';
	import type { Snippet } from 'svelte';

	export type AcerolaToggleGroupValue = string | string[] | undefined;

	export type AcerolaToggleGroupProps = {
		state?: {
			value?: AcerolaToggleGroupValue;
		};
		events?: {
			onValueChange?: (value: AcerolaToggleGroupValue) => void;
		};
		config?: {
			type?: ToggleGroupPrimitive.RootProps['type'];
		};
		ui?: Pick<ToggleGroupPrimitive.RootProps, 'class'> & {
			variant?: 'default' | 'outline';
			size?: 'default' | 'sm' | 'lg';
			/// Espaço (na escala do Tailwind, ex.: `4` = 1rem) entre os itens — default `0`
			/// (itens colados, cantos arredondados só nas pontas do grupo, como um segmented
			/// control). Qualquer valor > 0 desliga esse visual "colado" e cada item volta a
			/// ter cantos totalmente arredondados (ver `group-data-[spacing=0]` nas classes de
			/// `toggle-group-item.svelte`) — necessário sempre que os itens têm um gap visual
			/// real entre si, senão o item nasce com um lado sem arredondar.
			spacing?: number;
		};
		children: Snippet;
	};
</script>

<script lang="ts">
	import { untrack } from 'svelte';
	import * as ToggleGroup from '$lib/components/ui/toggle-group/index.js';

	let { children, config, events, state: control, ui }: AcerolaToggleGroupProps = $props();

	let value: AcerolaToggleGroupValue = $state();
	let lastValue: AcerolaToggleGroupValue = $state();

	const type = $derived(config?.type ?? 'single');
	const variant = $derived(ui?.variant ?? 'default');
	const size = $derived(ui?.size ?? 'default');
	const spacing = $derived(ui?.spacing ?? 0);

	$effect(() => {
		if (control?.value === undefined) return;

		const nextValue = control.value;

		if (nextValue !== untrack(() => lastValue)) {
			value = nextValue;
			lastValue = nextValue;
		}
	});

	$effect(() => {
		if (value === lastValue) return;

		lastValue = value;
		events?.onValueChange?.(value);
	});
</script>

{#if type === 'multiple'}
	<ToggleGroup.Root
		bind:value={value as string[] | undefined}
		type="multiple"
		{variant}
		{size}
		{spacing}
		class={ui?.class}
	>
		{@render children()}
	</ToggleGroup.Root>
{:else}
	<ToggleGroup.Root
		bind:value={value as string | undefined}
		type="single"
		{variant}
		{size}
		{spacing}
		class={ui?.class}
	>
		{@render children()}
	</ToggleGroup.Root>
{/if}
