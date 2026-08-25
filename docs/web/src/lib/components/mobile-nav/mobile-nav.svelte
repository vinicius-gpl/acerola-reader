<script lang="ts">
	import { Dialog } from 'bits-ui';
	import Sidebar from '$lib/components/sidebar/sidebar.svelte';
	import type { SidebarGroup } from '$lib/content/docs';

	let {
		open = $bindable(false),
		groups,
		activeSlug
	}: { open: boolean; groups: SidebarGroup[]; activeSlug: string } = $props();
</script>

<Dialog.Root bind:open>
	<Dialog.Portal>
		<Dialog.Overlay class="fixed inset-0 z-40 bg-black/40 md:hidden" />
		<Dialog.Content
			class="fixed inset-y-0 left-0 z-50 w-72 overflow-y-auto border-r border-border bg-background p-4 md:hidden"
		>
			<Dialog.Title class="sr-only">Menu</Dialog.Title>
			<Dialog.Description class="sr-only">Navegação da documentação</Dialog.Description>
			<Sidebar {groups} {activeSlug} />
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>
