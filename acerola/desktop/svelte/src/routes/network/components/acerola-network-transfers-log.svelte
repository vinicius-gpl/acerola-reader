<script module lang="ts">
	import type { TransferLogEntry } from '$lib/hooks/store/use-network-sync.svelte';

	export type NetworkTransfersLogProps = {
		data: {
			entries: TransferLogEntry[];
			peerLabel: (peerId: string) => string;
		};
	};
</script>

<script lang="ts">
	import AlertCircleIcon from '@lucide/svelte/icons/alert-circle';
	import CheckIcon from '@lucide/svelte/icons/check';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
	import { m } from '$lib/paraglide/messages';
	import { autoAnimateList } from '$lib/utils/auto-animate.utils';

	let { data }: NetworkTransfersLogProps = $props();

	// No teardown da story (Storybook + vitest browser mode), o efeito reativo deste
	// template roda mais uma vez com `data` já undefined antes do componente ser
	// destruído de fato — sem o fallback aqui isso vaza como unhandled error e derruba
	// a suíte mesmo com todos os asserts passando.
	let entries = $derived(data?.entries ?? []);

	type EntryMessageByStatus = Partial<
		Record<TransferLogEntry['status'], (entry: TransferLogEntry) => string>
	>;

	// Agrupada por kind e depois por status (em vez de um switch sobre a string
	// concatenada "kind:status") pra deixar visível quais combinações existem de fato —
	// nem todo kind tem "progress", por exemplo.
	const messageByKind = $derived<Record<TransferLogEntry['kind'], EntryMessageByStatus>>({
		history: {
			started: (entry) =>
				m['pages.network.transfers.history_started']({ peer: data.peerLabel(entry.message) }),
			complete: (entry) =>
				m['pages.network.transfers.history_complete']({ peer: data.peerLabel(entry.message) }),
			error: (entry) => m['pages.network.transfers.history_error']({ msg: entry.message })
		},
		files: {
			started: (entry) =>
				m['pages.network.transfers.files_started']({ peer: data.peerLabel(entry.message) }),
			progress: (entry) => m['pages.network.transfers.files_progress']({ item: entry.message }),
			complete: (entry) =>
				m['pages.network.transfers.files_complete']({ peer: data.peerLabel(entry.message) }),
			error: (entry) => m['pages.network.transfers.files_error']({ msg: entry.message })
		},
		comic: {
			started: (entry) =>
				m['pages.network.transfers.comic_started']({ peer: data.peerLabel(entry.message) }),
			progress: (entry) => m['pages.network.transfers.comic_progress']({ item: entry.message }),
			complete: (entry) =>
				m['pages.network.transfers.comic_complete']({ peer: data.peerLabel(entry.message) }),
			error: (entry) => m['pages.network.transfers.comic_error']({ msg: entry.message })
		}
	});

	function describeEntry(entry: TransferLogEntry): string {
		return messageByKind[entry.kind][entry.status]?.(entry) ?? entry.message;
	}
</script>

<div class="rounded-2xl border border-border/40 bg-card/50 p-4 backdrop-blur-sm">
	{#if entries.length === 0}
		<p class="p-4 text-center text-sm text-muted-foreground">
			{m['pages.network.transfers.empty']()}
		</p>
	{:else}
		<ul class="max-h-80 space-y-1 overflow-y-auto" use:autoAnimateList>
			{#each entries as entry (entry.id)}
				<li class="flex items-center gap-3 rounded-lg px-3 py-2 text-sm hover:bg-muted/50">
					{#if entry.status === 'error'}
						<AlertCircleIcon size={14} class="shrink-0 text-destructive" />
					{:else if entry.status === 'complete'}
						<CheckIcon size={14} class="shrink-0 text-chart-3" />
					{:else}
						<RefreshCwIcon size={14} class="shrink-0 animate-spin text-muted-foreground" />
					{/if}

					<span class="flex-1 truncate text-foreground">{describeEntry(entry)}</span>
					<span class="shrink-0 text-xs text-muted-foreground">
						{new Date(entry.timestamp).toLocaleTimeString()}
					</span>
				</li>
			{/each}
		</ul>
	{/if}
</div>
