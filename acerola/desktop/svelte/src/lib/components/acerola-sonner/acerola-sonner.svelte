<script lang="ts">
	import { Toaster as AcerolaToaster } from '$lib/components/ui/sonner/index';
	import type { ToasterProps as SonnerProps } from 'svelte-sonner';

	export type AcerolaSonnerProps = {
		ui?: SonnerProps;
	};

	let { ui }: AcerolaSonnerProps = $props();
</script>

<AcerolaToaster {...ui} />
