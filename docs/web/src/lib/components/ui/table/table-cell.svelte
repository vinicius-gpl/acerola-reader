<script lang="ts">
	import { cn, type WithElementRef } from '$lib/cn.util';
	import type { HTMLAttributes } from 'svelte/elements';

	let {
		ref = $bindable(null),
		class: className,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLTableCellElement>> = $props();
</script>

<td
	bind:this={ref}
	data-slot="table-cell"
	class={cn('px-4 py-2.5 align-top text-foreground', className)}
	{...restProps}
>
	{@render children?.()}
</td>
