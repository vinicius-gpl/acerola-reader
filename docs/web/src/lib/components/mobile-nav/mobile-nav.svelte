<script lang="ts">
	import * as Sheet from '$lib/components/ui/sheet/index.js';
	import Sidebar from '$lib/components/sidebar/sidebar.svelte';
	import type { SidebarGroup } from '$lib/content/docs';

	let {
		open = $bindable(false),
		groups,
		activeSlug
	}: { open: boolean; groups: SidebarGroup[]; activeSlug: string } = $props();
</script>

<Sheet.Root bind:open>
	<Sheet.Content side="left" class="w-72 overflow-y-auto p-4 md:hidden">
		<Sheet.Header class="sr-only">
			<Sheet.Title>Menu</Sheet.Title>
			<Sheet.Description>Navegação da documentação</Sheet.Description>
		</Sheet.Header>
		<Sidebar {groups} {activeSlug} />
	</Sheet.Content>
</Sheet.Root>
