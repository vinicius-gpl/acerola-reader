<script lang="ts">
	import { onMount } from 'svelte';
	import { useReaderNavigation } from '$lib/hooks/store/use-reader-navigation.svelte';

	let {
		onReady
	}: {
		onReady: (hook: ReturnType<typeof useReaderNavigation>) => void;
	} = $props();

	const hook = useReaderNavigation();

	onMount(() => {
		onReady(hook);
	});
</script>
