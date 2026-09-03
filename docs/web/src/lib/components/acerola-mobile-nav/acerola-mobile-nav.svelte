<script lang="ts">
	import * as Sheet from '$lib/components/ui/sheet/index';
	import AcerolaNavControls from '$lib/components/acerola-nav-controls/acerola-nav-controls.svelte';
	import AcerolaSidebar from '$lib/components/acerola-sidebar/acerola-sidebar.svelte';
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
			<AcerolaSidebar {groups} {activeSlug} />
		</div>
		<div class="shrink-0 border-t border-border p-3">
			<AcerolaNavControls />
		</div>
	</Sheet.Content>
</Sheet.Root>
