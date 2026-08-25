<script lang="ts" module>
	import CircleAlertIcon from '@lucide/svelte/icons/circle-alert';
	import InfoIcon from '@lucide/svelte/icons/info';
	import LightbulbIcon from '@lucide/svelte/icons/lightbulb';
	import OctagonAlertIcon from '@lucide/svelte/icons/octagon-alert';
	import type { Component } from 'svelte';

	export type CalloutType = 'note' | 'tip' | 'caution' | 'danger';

	const CALLOUT_CONFIG: Record<CalloutType, { icon: Component; class: string }> = {
		note: { icon: InfoIcon, class: 'border-primary/40 bg-primary/5 text-primary' },
		tip: { icon: LightbulbIcon, class: 'border-chart-4/40 bg-chart-4/5 text-chart-4' },
		caution: { icon: CircleAlertIcon, class: 'border-chart-2/40 bg-chart-2/5 text-chart-2' },
		danger: {
			icon: OctagonAlertIcon,
			class: 'border-destructive/40 bg-destructive/5 text-destructive'
		}
	};
</script>

<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		type = 'note',
		title,
		children
	}: { type?: CalloutType; title?: string; children: Snippet } = $props();

	const config = $derived(CALLOUT_CONFIG[type]);
</script>

<aside class={['my-6 rounded-lg border-l-4 p-4', config.class]}>
	<div class="flex items-center gap-2 font-semibold">
		<config.icon size={18} />
		<span class="text-foreground">{title ?? type}</span>
	</div>
	<div class="mt-2 text-sm text-foreground [&>p]:my-0">
		{@render children()}
	</div>
</aside>
