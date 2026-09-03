<script module lang="ts">
	export type AcerolaSettingsHeaderProps = {
		data: {
			title: string;
		};
		events: {
			onBack: () => void;
		};
	};
</script>

<script lang="ts">
	import AcerolaButtonIcon from '$lib/components/acerola-button/acerola-button-icon.svelte';
	import ArrowLeft from '@lucide/svelte/icons/arrow-left';

	let { data, events }: AcerolaSettingsHeaderProps = $props();
</script>

<div class="flex items-center gap-3">
	<AcerolaButtonIcon events={{ onClick: events.onBack }} ui={{ size: 'sm' }}>
		<ArrowLeft size={18} />
	</AcerolaButtonIcon>
	<h1 class="truncate text-2xl font-bold tracking-tight text-foreground">
		{data.title}
	</h1>
</div>
