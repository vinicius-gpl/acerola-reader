<script lang="ts">
	import { cn, type WithElementRef } from '$lib/cn.util';
	import type { HTMLAttributes } from 'svelte/elements';

	let {
		ref = $bindable(null),
		class: className,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLTableElement>> = $props();
</script>

<div
	data-slot="table-container"
	class="relative w-full overflow-x-auto rounded-lg border border-border"
>
	<table
		bind:this={ref}
		data-slot="table"
		class={cn('w-full caption-bottom border-collapse text-sm', className)}
		{...restProps}
	>
		{@render children?.()}
	</table>
</div>
