<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaSidebar from './acerola-sidebar.svelte';
	import HouseIcon from '@lucide/svelte/icons/house';
	import HistoryIcon from '@lucide/svelte/icons/history';
	import Share2Icon from '@lucide/svelte/icons/share-2';
	import SettingsIcon from '@lucide/svelte/icons/settings';
	import AcerolaLogo from '$lib/assets/icons/acerola.svg?component';

	const items = [
		{ href: '/home', label: 'Home', icon: HouseIcon },
		{ href: '/history', label: 'History', icon: HistoryIcon },
		{ href: '/network', label: 'Network', icon: Share2Icon },
		{ href: '/config', label: 'Settings', icon: SettingsIcon }
	];

	const { Story } = defineMeta({
		component: AcerolaSidebar,
		title: 'Components/AcerolaSidebar'
	});
</script>

<Story name="Default">
	{#snippet children()}
		<div class="flex h-96 bg-surface">
			<AcerolaSidebar data={{ items }} />
		</div>
	{/snippet}
</Story>

<Story name="With Brand">
	{#snippet children()}
		<div class="flex h-96 bg-surface">
			<AcerolaSidebar data={{ items }}>
				{#snippet brand()}
					<AcerolaLogo class="h-full w-full" />
				{/snippet}
			</AcerolaSidebar>
		</div>
	{/snippet}
</Story>
