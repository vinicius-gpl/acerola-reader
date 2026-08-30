<script lang="ts">
	import { cn, type WithElementRef } from '$lib/cn.util';
	import type { HTMLAttributes } from 'svelte/elements';

	let {
		ref = $bindable(null),
		class: className,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLTableSectionElement>> = $props();
</script>

<thead
	bind:this={ref}
	data-slot="table-header"
	class={cn('bg-muted [&_tr]:border-b [&_tr]:border-border', className)}
	{...restProps}
>
	{@render children?.()}
</thead>
