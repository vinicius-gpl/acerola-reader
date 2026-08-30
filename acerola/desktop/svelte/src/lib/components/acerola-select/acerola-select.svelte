<script module lang="ts">
	export type AcerolaSelectOption = {
		value: string;
		label: string;
		color?: number;
	};

	export type AcerolaSelectProps = {
		data: {
			options: AcerolaSelectOption[];
		};
		state?: {
			value?: string;
		};
		events?: {
			onValueChange?: (value: string) => void;
		};
		ui?: {
			class?: string;
			placeholder?: string;
		};
	};
</script>

<script lang="ts">
	import { untrack } from 'svelte';
	import { Select, SelectContent, SelectItem, SelectTrigger } from '$lib/components/ui/select';
	import { m } from '$lib/paraglide/messages';
	import { cn } from '$lib/utils/cn.utils';

	let { data, state: control, events, ui }: AcerolaSelectProps = $props();

	let value = $state('');
	let lastValue = $state('');

	const placeholder = $derived(ui?.placeholder ?? m['components.select.placeholder']());

	let selectedOption = $derived(data.options.find((it) => it.value === value));
	let selectedLabel = $derived(selectedOption?.label ?? placeholder);

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

<Select type="single" bind:value>
	<SelectTrigger
		class={cn(
			'h-10 w-auto min-w-48 justify-between border-surface/30 bg-surface/20 transition-all hover:bg-surface/40',
			ui?.class
		)}
	>
		<div class="flex items-center gap-2">
			{#if selectedOption?.color != null}
				{@const hex = (selectedOption.color & 0xffffff).toString(16).padStart(6, '0')}
				<div
					class="flex items-center gap-2 rounded-md border px-2 py-0.5 shadow-sm"
					style="background-color: #{hex}15; border-color: #{hex}30; color: #{hex}"
				>
					<div class="h-2 w-2 rounded-full shadow-sm" style="background-color: #{hex}"></div>
					<span class="text-xs font-bold tracking-widest uppercase">{selectedLabel}</span>
				</div>
			{:else}
				<span class="text-sm font-medium text-foreground">{selectedLabel}</span>
			{/if}
		</div>
	</SelectTrigger>

	<SelectContent>
		{#each data.options as option (option.value)}
			<SelectItem value={option.value} label={option.label} class="cursor-pointer">
				<div class="flex items-center gap-2 py-0.5">
					{#if option.color != null}
						{@const hex = (option.color & 0xffffff).toString(16).padStart(6, '0')}
						<div
							class="flex items-center gap-2 rounded-md px-2 py-0.5 transition-colors"
							style="background-color: #{hex}15; color: #{hex}"
						>
							<div class="h-2 w-2 rounded-full" style="background-color: #{hex}"></div>
							<span class="text-xs font-bold tracking-widest uppercase">{option.label}</span>
						</div>
					{:else}
						<span class="text-sm font-medium">{option.label}</span>
					{/if}
				</div>
			</SelectItem>
		{/each}
	</SelectContent>
</Select>
