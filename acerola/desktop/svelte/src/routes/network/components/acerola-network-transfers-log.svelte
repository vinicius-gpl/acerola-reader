<script module lang="ts">
	import type { TransferLogEntry } from '$lib/hooks/store/use-network-sync.svelte';

	export type NetworkTransfersLogProps = {
		data: {
			entries: TransferLogEntry[];
			peerLabel: (peerId: string) => string;
		};
		events?: {
			onRefresh?: () => void;
			onClear?: () => void;
		};
	};
</script>

<script lang="ts">
	import AlertCircleIcon from '@lucide/svelte/icons/alert-circle';
	import CheckIcon from '@lucide/svelte/icons/check';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
	import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
	import ArrowLeftRightIcon from '@lucide/svelte/icons/arrow-left-right';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';
	import { m } from '$lib/paraglide/messages';
	import AcerolaButtonIcon from '$lib/components/acerola-button/acerola-button-icon.svelte';
	import AcerolaAlertDialog from '$lib/components/acerola-alert-dialog/acerola-alert-dialog.svelte';
	import AcerolaAccordionCard from '$lib/components/acerola-accordion-card/acerola-accordion-card.svelte';
	import AcerolaHeroButton from '$lib/components/acerola-hero-button/acerola-hero-button.svelte';

	let { data, events }: NetworkTransfersLogProps = $props();

	// No teardown da story (Storybook + vitest browser mode), o efeito reativo deste
	// template roda mais uma vez com `data` já undefined antes do componente ser
	// destruído de fato — sem o fallback aqui isso vaza como unhandled error e derruba
	// a suíte mesmo com todos os asserts passando.
	let entries = $derived(data?.entries ?? []);

	// Fechado por padrão, expande em linha — mesmo componente e mesma ideia do card de
	// Configurações de Relay logo acima nessa tela, em vez de um dialog à parte.
	let expanded = $state(false);

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
		historyEntry: {
			started: (entry) =>
				m['pages.network.transfers.history_entry_started']({ peer: data.peerLabel(entry.message) }),
			complete: (entry) =>
				m['pages.network.transfers.history_entry_complete']({
					peer: data.peerLabel(entry.message)
				}),
			error: (entry) => m['pages.network.transfers.history_entry_error']({ msg: entry.message })
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
		const base = messageByKind[entry.kind][entry.status]?.(entry) ?? entry.message;
		// Sem toast/notificação própria pra isso (ver `use-network-sync.svelte.ts`) — o
		// conflito só precisa aparecer aqui, na linha da sessão que já ia ser mostrada de
		// qualquer forma, um total por sessão em vez de uma notificação por capítulo.
		if (entry.status === 'complete' && entry.conflicts) {
			return `${base} ${m['pages.network.transfers.conflicts_suffix']({ count: entry.conflicts })}`;
		}
		return base;
	}

	// Resumo mostrado no cabeçalho do accordion fechado — a entrada mais recente
	// (`entries[0]`, ver `use-network-sync.svelte.ts`) ou o texto de vazio.
	let summary = $derived(
		entries.length > 0 ? describeEntry(entries[0]) : m['pages.network.transfers.empty']()
	);

	// Fundo do círculo do ícone SÓLIDO tintado pelo status (o bg contorna o ícone, que fica
	// numa cor de foreground neutra por cima) — não o `bg-muted` genérico padrão do
	// AcerolaHeroButton.
	function iconBackgroundClass(entry: TransferLogEntry): string {
		if (entry.status === 'error') return 'bg-destructive text-destructive-foreground';
		if (entry.status === 'complete') return 'bg-chart-3 text-primary-foreground';
		return 'bg-muted text-muted-foreground';
	}
</script>

<AcerolaAccordionCard
	data={{ title: m['pages.network.transfers.title'](), description: summary }}
	state={{ expanded }}
	events={{ onToggle: () => (expanded = !expanded) }}
	ui={{ iconClass: 'bg-accent-hero text-accent-hero-foreground' }}
>
	{#snippet icon()}
		<ArrowLeftRightIcon size={20} />
	{/snippet}

	<div class="flex items-center justify-end gap-1">
		<AcerolaButtonIcon
			events={{ onClick: () => events?.onRefresh?.() }}
			ui={{
				variant: 'ghost',
				tone: 'accent',
				class: 'size-8',
				title: m['pages.network.transfers.refresh'](),
				'aria-label': m['pages.network.transfers.refresh']()
			}}
		>
			<RefreshCwIcon size={14} />
		</AcerolaButtonIcon>

		{#if entries.length > 0}
			<AcerolaAlertDialog
				data={{
					title: m['pages.network.transfers.clear.title'](),
					description: m['pages.network.transfers.clear.desc'](),
					cancelText: m['pages.network.transfers.clear.cancel'](),
					actionText: m['pages.network.transfers.clear.confirm']()
				}}
				ui={{ variant: 'destructive' }}
				events={{ onAction: () => events?.onClear?.() }}
			>
				<AcerolaButtonIcon
					ui={{
						variant: 'ghost',
						tone: 'destructive',
						class: 'size-8',
						title: m['pages.network.transfers.clear.button'](),
						'aria-label': m['pages.network.transfers.clear.button']()
					}}
				>
					<Trash2Icon size={14} />
				</AcerolaButtonIcon>
			</AcerolaAlertDialog>
		{/if}
	</div>

	{#if entries.length === 0}
		<p class="p-4 text-center text-sm text-muted-foreground">
			{m['pages.network.transfers.empty']()}
		</p>
	{:else}
		<div class="max-h-[28rem] space-y-2 overflow-y-auto pr-1">
			{#each entries as entry (entry.id)}
				<AcerolaHeroButton
					data={{
						title: describeEntry(entry),
						description: new Date(entry.timestamp).toLocaleTimeString()
					}}
					ui={{ iconClass: iconBackgroundClass(entry) }}
				>
					{#snippet icon()}
						{#if entry.status === 'error'}
							<AlertCircleIcon size={20} />
						{:else if entry.status === 'complete'}
							<CheckIcon size={20} />
						{:else if entry.status === 'started'}
							<RefreshCwIcon size={20} class="animate-spin" />
						{:else}
							<ArrowRightIcon size={20} />
						{/if}
					{/snippet}
				</AcerolaHeroButton>
			{/each}
		</div>
	{/if}
</AcerolaAccordionCard>
