<script module lang="ts">
	import type { Snippet } from 'svelte';

	export type AcerolaCommandProps = {
		state?: {
			value?: string;
		};
		events?: {
			onValueChange?: (value: string) => void;
		};
		children: Snippet;
	};
</script>

<script lang="ts">
	import { untrack } from 'svelte';
	import * as Command from '$lib/components/ui/command/index';

	let { children, events, state: control }: AcerolaCommandProps = $props();

	let commandValue = $state('');
	let lastValue = $state('');

	$effect(() => {
		if (control?.value === undefined) return;

		const nextValue = control.value;

		if (nextValue !== untrack(() => lastValue)) {
			commandValue = nextValue;
			lastValue = nextValue;
		}
	});

	$effect(() => {
		if (commandValue === lastValue) return;

		lastValue = commandValue;
		events?.onValueChange?.(commandValue);
	});
</script>

<Command.Root bind:value={commandValue}>
	{@render children()}
</Command.Root>
