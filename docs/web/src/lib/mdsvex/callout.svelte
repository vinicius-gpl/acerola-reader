<script lang="ts" module>
	import CircleAlertIcon from '@lucide/svelte/icons/circle-alert';
	import InfoIcon from '@lucide/svelte/icons/info';
	import LightbulbIcon from '@lucide/svelte/icons/lightbulb';
	import OctagonAlertIcon from '@lucide/svelte/icons/octagon-alert';
	import type { AlertVariant } from '$lib/components/ui/alert/index.js';
	import type { Component } from 'svelte';

	export type CalloutType = 'note' | 'tip' | 'caution' | 'danger';

	const CALLOUT_CONFIG: Record<CalloutType, { icon: Component; variant: AlertVariant }> = {
		note: { icon: InfoIcon, variant: 'note' },
		tip: { icon: LightbulbIcon, variant: 'tip' },
		caution: { icon: CircleAlertIcon, variant: 'caution' },
		danger: { icon: OctagonAlertIcon, variant: 'destructive' }
	};
</script>

<script lang="ts">
	import * as Alert from '$lib/components/ui/alert/index.js';
	import type { Snippet } from 'svelte';

	let {
		type = 'note',
		title,
		children
	}: { type?: CalloutType; title?: string; children: Snippet } = $props();

	const config = $derived(CALLOUT_CONFIG[type]);
</script>

<Alert.Root variant={config.variant} class="my-6 py-3">
	<config.icon size={18} />
	<Alert.Title>{title ?? type}</Alert.Title>
	<Alert.Description class="text-foreground [&>p]:my-0">
		{@render children()}
	</Alert.Description>
</Alert.Root>
