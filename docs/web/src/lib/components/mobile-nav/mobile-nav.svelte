<script lang="ts">
	import * as Sheet from '$lib/components/ui/sheet/index.js';
	import NavControls from '$lib/components/nav-controls/nav-controls.svelte';
	import Sidebar from '$lib/components/sidebar/sidebar.svelte';
	import type { SidebarGroup } from '$lib/content/docs';

	let {
		open = $bindable(false),
		groups,
		activeSlug
	}: { open: boolean; groups: SidebarGroup[]; activeSlug: string } = $props();
</script>

<Sheet.Root bind:open>
	<Sheet.Content side="left" class="flex w-72 flex-col gap-0 p-0 md:hidden">
		<Sheet.Header class="sr-only">
			<Sheet.Title>Menu</Sheet.Title>
			<Sheet.Description>Navegação da documentação</Sheet.Description>
		</Sheet.Header>
		<div class="min-h-0 flex-1 overflow-y-auto p-4">
			<Sidebar {groups} {activeSlug} />
		</div>
		<div class="shrink-0 border-t border-border p-3">
			<NavControls />
		</div>
	</Sheet.Content>
</Sheet.Root>
