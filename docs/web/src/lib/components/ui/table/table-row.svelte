<script lang="ts">
	import { cn, type WithElementRef } from '$lib/cn.util';
	import type { HTMLAttributes } from 'svelte/elements';

	let {
		ref = $bindable(null),
		class: className,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLTableRowElement>> = $props();
</script>

<tr
	bind:this={ref}
	data-slot="table-row"
	class={cn('border-b border-border transition-colors hover:bg-accent/50', className)}
	{...restProps}
>
	{@render children?.()}
</tr>
