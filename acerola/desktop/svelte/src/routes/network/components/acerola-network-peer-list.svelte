<script module lang="ts">
	export type DisplayPeer = { peerId: string; deviceName: string | null; connected: boolean };

	export type NetworkPeerListProps = {
		data: {
			peers: DisplayPeer[];
			addrFor: (peerId: string) => number[] | undefined;
			statusLabel: (peer: DisplayPeer) => string;
			isSyncing: (peerId: string, kind: 'history' | 'files') => boolean;
		};
		events: {
			onSyncHistory: (peerId: string) => void;
			onSyncFiles: (peerId: string) => void;
			onSyncAll: (peerId: string) => void;
			onBrowseLibrary: (peerId: string) => void;
			onRemove: (peer: DisplayPeer) => void;
		};
	};
</script>

<script lang="ts">
	import UsersIcon from '@lucide/svelte/icons/users';
	import MonitorIcon from '@lucide/svelte/icons/monitor';
	import MoreVerticalIcon from '@lucide/svelte/icons/more-vertical';
	import HistoryIcon from '@lucide/svelte/icons/history';
	import FolderSyncIcon from '@lucide/svelte/icons/folder-sync';
	import BookOpenIcon from '@lucide/svelte/icons/book-open';
	import UserMinusIcon from '@lucide/svelte/icons/user-minus';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
	import AcerolaHeroButton from '$lib/components/acerola-hero-button/acerola-hero-button.svelte';
	import AcerolaButton from '$lib/components/acerola-button/acerola-button.svelte';
	import AcerolaPopover from '$lib/components/acerola-popover/acerola-popover.svelte';
	import { m } from '$lib/paraglide/messages';
	import { shortId } from '$lib/utils/connection-code.utils';

	let { data, events }: NetworkPeerListProps = $props();

	let openPeerMenuId = $state<string | null>(null);

	// No teardown da story (Storybook + vitest browser mode), o efeito reativo deste
	// template roda mais uma vez com `data` já undefined antes do componente ser
	// destruído de fato — sem o fallback aqui isso vaza como unhandled error e derruba
	// a suíte mesmo com todos os asserts passando.
	let peers = $derived(data?.peers ?? []);
</script>

<section class="space-y-4">
	<div
		class="flex items-center gap-3 text-xs font-bold tracking-widest text-muted-foreground uppercase"
	>
		<UsersIcon size={16} />
		{m['pages.network.peers.title']()}
	</div>

	<div class="grid gap-4">
		{#if peers.length === 0}
			<div
				class="rounded-xl border border-dashed border-border/40 p-8 text-center text-sm text-muted-foreground"
			>
				{m['pages.network.peers.empty']()}
			</div>
		{:else}
			{#each peers as peer (peer.peerId)}
				{@const addrs = data.addrFor(peer.peerId)}
				{@const historySyncing = data.isSyncing(peer.peerId, 'history')}
				{@const filesSyncing = data.isSyncing(peer.peerId, 'files')}
				{@const anySyncing = historySyncing || filesSyncing}

				<AcerolaHeroButton
					data={{
						title: peer.deviceName ?? shortId(peer.peerId),
						description: data.statusLabel(peer)
					}}
				>
					{#snippet icon()}
						<div class="relative flex h-full w-full items-center justify-center">
							<MonitorIcon size={22} />
							<span
								aria-hidden="true"
								class="absolute right-0 bottom-0 size-3 rounded-full border-2 border-muted {peer.connected
									? 'bg-chart-3'
									: 'bg-muted-foreground/40'}"
							></span>
						</div>
					{/snippet}

					{#snippet action()}
						<div class="flex items-center gap-2">
							<AcerolaPopover
								state={{ open: openPeerMenuId === peer.peerId }}
								events={{
									onOpenChange: (open) => (openPeerMenuId = open ? peer.peerId : null)
								}}
								ui={{
									align: 'end',
									contentClass:
										'w-56 overflow-hidden rounded-2xl border-border/40 bg-card/95 p-1.5 shadow-2xl backdrop-blur-md'
								}}
							>
								{#snippet trigger()}
									<span
										class="text-overlay flex size-8 items-center justify-center rounded-xl transition-colors hover:bg-surface/60 hover:text-primary"
									>
										<MoreVerticalIcon size={16} />
										<span class="sr-only">{m['pages.network.peers.more_actions']()}</span>
									</span>
								{/snippet}

								{#snippet content()}
									<div class="flex flex-col gap-0.5">
										<AcerolaButton
											events={{
												onClick: () => {
													openPeerMenuId = null;
													events.onSyncHistory(peer.peerId);
												}
											}}
											ui={{
												variant: 'ghost',
												disabled: !addrs || historySyncing,
												class:
													'h-9 w-full justify-start gap-2.5 rounded-xl px-2.5 text-sm font-medium'
											}}
										>
											{#if historySyncing}
												<RefreshCwIcon size={16} class="shrink-0 animate-spin" />
											{:else}
												<HistoryIcon size={16} class="shrink-0" />
											{/if}
											{m['pages.network.peers.sync_history']()}
										</AcerolaButton>

										<AcerolaButton
											events={{
												onClick: () => {
													openPeerMenuId = null;
													events.onSyncFiles(peer.peerId);
												}
											}}
											ui={{
												variant: 'ghost',
												disabled: !addrs || filesSyncing,
												class:
													'h-9 w-full justify-start gap-2.5 rounded-xl px-2.5 text-sm font-medium'
											}}
										>
											{#if filesSyncing}
												<RefreshCwIcon size={16} class="shrink-0 animate-spin" />
											{:else}
												<FolderSyncIcon size={16} class="shrink-0" />
											{/if}
											{m['pages.network.peers.sync_files']()}
										</AcerolaButton>

										<AcerolaButton
											events={{
												onClick: () => {
													openPeerMenuId = null;
													events.onBrowseLibrary(peer.peerId);
												}
											}}
											ui={{
												variant: 'ghost',
												disabled: !addrs,
												class:
													'h-9 w-full justify-start gap-2.5 rounded-xl px-2.5 text-sm font-medium'
											}}
										>
											<BookOpenIcon size={16} class="shrink-0" />
											{m['pages.network.peers.browse_library']()}
										</AcerolaButton>

										<div class="my-1 h-px bg-border/60"></div>

										<AcerolaButton
											events={{
												onClick: () => {
													openPeerMenuId = null;
													events.onRemove(peer);
												}
											}}
											ui={{
												variant: 'ghost',
												class:
													'h-9 w-full justify-start gap-2.5 rounded-xl px-2.5 text-sm font-medium text-destructive hover:bg-destructive/10 hover:text-destructive'
											}}
										>
											<UserMinusIcon size={16} class="shrink-0" />
											{m['pages.network.peers.remove']()}
										</AcerolaButton>
									</div>
								{/snippet}
							</AcerolaPopover>

							<AcerolaButton
								events={{ onClick: () => events.onSyncAll(peer.peerId) }}
								ui={{ size: 'sm', class: 'gap-2', disabled: !addrs || anySyncing }}
							>
								<RefreshCwIcon size={14} class={anySyncing ? 'animate-spin' : ''} />
								{m['pages.network.peers.sync_all']()}
							</AcerolaButton>
						</div>
					{/snippet}
				</AcerolaHeroButton>

				{#if !addrs}
					<p class="-mt-2 px-2 text-xs text-muted-foreground">
						{m['pages.network.peers.offline_hint']()}
					</p>
				{/if}
			{/each}
		{/if}
	</div>
</section>
