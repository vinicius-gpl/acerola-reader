<script lang="ts">
	import { Tabs as TabsPrimitive } from 'bits-ui';
	import { cn } from '$lib/utils/cn.utils.js';

	let {
		ref = $bindable(null),
		class: className,
		value,
		children,
		...restProps
	}: TabsPrimitive.TriggerProps = $props();
</script>

<TabsPrimitive.Trigger
	bind:ref
	data-slot="tabs-trigger"
	class={cn(
		'inline-flex items-center justify-center outline-none disabled:pointer-events-none disabled:opacity-50',
		className
	)}
	{value}
	{...restProps}
>
	{@render children?.()}
</TabsPrimitive.Trigger>
