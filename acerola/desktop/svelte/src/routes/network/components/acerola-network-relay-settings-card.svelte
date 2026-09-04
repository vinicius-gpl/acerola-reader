<script module lang="ts">
	export type NetworkRelaySettingsCardData = {
		acerolaRelayUrl: string;
		useAcerolaRelay: boolean;
		useIrohPublicNetwork: boolean;
		customRelayUrls: string[];
		irohRelayUrls: string[];
		hasIrohServicesTicket: boolean;
	};

	export type NetworkRelaySettingsCardProps = {
		data: NetworkRelaySettingsCardData | undefined;
		events: {
			onToggleAcerolaRelay: (value: boolean) => void;
			onToggleIrohPublicNetwork: (value: boolean) => void;
			onAddCustomRelayUrl: (url: string) => void;
			onRemoveCustomRelayUrl: (url: string) => void;
			onAddIrohRelayUrl: (url: string) => void;
			onRemoveIrohRelayUrl: (url: string) => void;
			onSetIrohServicesTicket: (ticket: string) => Promise<void>;
			onClearIrohServicesTicket: () => Promise<void>;
		};
	};
</script>

<script lang="ts">
	import PlusIcon from '@lucide/svelte/icons/plus';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';
	import WifiIcon from '@lucide/svelte/icons/wifi';
	import AcerolaAccordionCard from '$lib/components/acerola-accordion-card/acerola-accordion-card.svelte';
	import AcerolaSwitch from '$lib/components/acerola-switch/acerola-switch.svelte';
	import AcerolaInput from '$lib/components/acerola-input/acerola-input.svelte';
	import AcerolaButton from '$lib/components/acerola-button/acerola-button.svelte';
	import AcerolaButtonIcon from '$lib/components/acerola-button/acerola-button-icon.svelte';
	import { m } from '$lib/paraglide/messages';

	let { data, events }: NetworkRelaySettingsCardProps = $props();

	// Fecha por padrão — a maioria dos usuários nunca precisa mexer aqui (relay do Acerola
	// já vem ligado por padrão), então não vale ocupar espaço da tela de Rede aberto.
	let expanded = $state(false);
	let customUrlDraft = $state('');
	let irohUrlDraft = $state('');
	let customUrlError = $state(false);
	let irohUrlError = $state(false);
	let ticketDraft = $state('');
	let ticketError = $state(false);
	let ticketSaving = $state(false);

	let safeData = $derived(
		data ?? {
			acerolaRelayUrl: '',
			useAcerolaRelay: false,
			useIrohPublicNetwork: false,
			customRelayUrls: [],
			irohRelayUrls: [],
			hasIrohServicesTicket: false
		}
	);

	const activeSourceCount = $derived(
		(safeData.useAcerolaRelay ? 1 : 0) +
			safeData.customRelayUrls.length +
			safeData.irohRelayUrls.length
	);

	// Espelha `RelaySettings::resolve` no backend: rede pública Iroh é exclusiva com as
	// demais fontes, e nenhuma fonte ativa cai em mDNS-only — não são combináveis entre si.
	const summary = $derived.by(() => {
		if (safeData.useIrohPublicNetwork) {
			return m['pages.network.relay_settings.summary_iroh_public']();
		}
		if (activeSourceCount === 0) {
			return m['pages.network.relay_settings.summary_mdns_only']();
		}
		return m['pages.network.relay_settings.summary_active']({ count: activeSourceCount });
	});

	function isValidUrl(value: string): boolean {
		try {
			const url = new URL(value);
			return url.protocol === 'http:' || url.protocol === 'https:';
		} catch {
			return false;
		}
	}

	function submitCustomUrl() {
		const trimmed = customUrlDraft.trim();
		if (!trimmed) return;
		if (!isValidUrl(trimmed)) {
			customUrlError = true;
			return;
		}
		customUrlError = false;
		events.onAddCustomRelayUrl(trimmed);
		customUrlDraft = '';
	}

	function submitIrohUrl() {
		const trimmed = irohUrlDraft.trim();
		if (!trimmed) return;
		if (!isValidUrl(trimmed)) {
			irohUrlError = true;
			return;
		}
		irohUrlError = false;
		events.onAddIrohRelayUrl(trimmed);
		irohUrlDraft = '';
	}

	async function submitTicket() {
		const trimmed = ticketDraft.trim();
		if (!trimmed || ticketSaving) return;

		ticketSaving = true;
		try {
			await events.onSetIrohServicesTicket(trimmed);
			ticketDraft = '';
			ticketError = false;
		} catch {
			ticketError = true;
		} finally {
			ticketSaving = false;
		}
	}

	async function removeTicket() {
		if (ticketSaving) return;
		ticketSaving = true;
		try {
			await events.onClearIrohServicesTicket();
		} finally {
			ticketSaving = false;
		}
	}
</script>

<AcerolaAccordionCard
	data={{ title: m['pages.network.relay_settings.title'](), description: summary }}
	state={{ expanded }}
	events={{ onToggle: () => (expanded = !expanded) }}
>
	{#snippet icon()}
		<WifiIcon size={20} />
	{/snippet}

	<div class="flex items-center justify-between gap-4">
		<div class="min-w-0">
			<p class="text-sm font-semibold text-foreground">
				{m['pages.network.relay_settings.use_acerola_relay']()}
			</p>
			<p class="text-xs text-muted-foreground">
				{m['pages.network.relay_settings.use_acerola_relay_desc']({
					url: safeData.acerolaRelayUrl
				})}
			</p>
		</div>
		<AcerolaSwitch
			state={{ checked: safeData.useAcerolaRelay }}
			events={{ onCheckedChange: events.onToggleAcerolaRelay }}
			ui={{ disabled: safeData.useIrohPublicNetwork }}
		/>
	</div>

	<div class="flex items-center justify-between gap-4">
		<div class="min-w-0">
			<p class="text-sm font-semibold text-foreground">
				{m['pages.network.relay_settings.use_iroh_public_network']()}
			</p>
			<p class="text-xs text-muted-foreground">
				{m['pages.network.relay_settings.use_iroh_public_network_desc']()}
			</p>
		</div>
		<AcerolaSwitch
			state={{ checked: safeData.useIrohPublicNetwork }}
			events={{ onCheckedChange: events.onToggleIrohPublicNetwork }}
			ui={{ disabled: !safeData.hasIrohServicesTicket }}
		/>
	</div>

	{#if safeData.useIrohPublicNetwork}
		<p class="text-xs text-muted-foreground italic">
			{m['pages.network.relay_settings.exclusive_note']()}
		</p>
	{/if}

	<div class="space-y-2 rounded-xl border border-border bg-background/50 p-3">
		<p class="text-xs font-bold tracking-widest text-muted-foreground uppercase">
			{m['pages.network.relay_settings.iroh_services_ticket.label']()}
		</p>
		<p class="text-xs text-muted-foreground">
			{safeData.hasIrohServicesTicket
				? m['pages.network.relay_settings.iroh_services_ticket.configured']()
				: m['pages.network.relay_settings.iroh_services_ticket.not_configured']()}
		</p>

		<div class="flex items-center gap-2">
			<AcerolaInput
				state={{ value: ticketDraft }}
				events={{
					onValueChange: (value) => {
						ticketDraft = value;
						ticketError = false;
					}
				}}
				ui={{
					type: 'password',
					placeholder: m['pages.network.relay_settings.iroh_services_ticket.placeholder'](),
					class: 'flex-1',
					disabled: ticketSaving
				}}
			/>
			<AcerolaButton
				events={{ onClick: submitTicket }}
				ui={{ size: 'sm', disabled: !ticketDraft.trim() || ticketSaving }}
			>
				{safeData.hasIrohServicesTicket
					? m['pages.network.relay_settings.iroh_services_ticket.replace_button']()
					: m['pages.network.relay_settings.iroh_services_ticket.save_button']()}
			</AcerolaButton>
			{#if safeData.hasIrohServicesTicket}
				<AcerolaButtonIcon
					events={{ onClick: removeTicket }}
					ui={{
						variant: 'ghost',
						class: 'size-8 text-muted-foreground hover:bg-destructive/10 hover:text-destructive',
						disabled: ticketSaving,
						'aria-label': m['pages.network.relay_settings.iroh_services_ticket.remove_button']()
					}}
				>
					<Trash2Icon size={14} />
				</AcerolaButtonIcon>
			{/if}
		</div>

		{#if ticketError}
			<p class="text-xs text-destructive">
				{m['pages.network.relay_settings.iroh_services_ticket.invalid']()}
			</p>
		{/if}

		<p class="text-xs text-muted-foreground italic">
			{m['pages.network.relay_settings.iroh_services_ticket.help']()}
		</p>
	</div>

	<div class="space-y-2" class:opacity-50={safeData.useIrohPublicNetwork}>
		<p class="text-xs font-bold tracking-widest text-muted-foreground uppercase">
			{m['pages.network.relay_settings.custom_relays.title']()}
		</p>

		{#each safeData.customRelayUrls as url (url)}
			<div
				class="flex items-center justify-between gap-3 rounded-xl border border-border bg-background/50 p-3"
			>
				<span class="min-w-0 flex-1 truncate text-sm text-foreground">{url}</span>
				<AcerolaButtonIcon
					events={{ onClick: () => events.onRemoveCustomRelayUrl(url) }}
					ui={{
						variant: 'ghost',
						class: 'size-8 text-muted-foreground hover:bg-destructive/10 hover:text-destructive',
						disabled: safeData.useIrohPublicNetwork,
						'aria-label': m['pages.network.relay_settings.custom_relays.remove']()
					}}
				>
					<Trash2Icon size={14} />
				</AcerolaButtonIcon>
			</div>
		{:else}
			<p class="text-xs text-muted-foreground">
				{m['pages.network.relay_settings.custom_relays.empty']()}
			</p>
		{/each}

		<div class="flex items-center gap-2">
			<AcerolaInput
				state={{ value: customUrlDraft }}
				events={{
					onValueChange: (value) => {
						customUrlDraft = value;
						customUrlError = false;
					}
				}}
				ui={{
					placeholder: m['pages.network.relay_settings.custom_relays.add_placeholder'](),
					class: 'flex-1',
					disabled: safeData.useIrohPublicNetwork
				}}
			/>
			<AcerolaButton
				events={{ onClick: submitCustomUrl }}
				ui={{ size: 'sm', disabled: !customUrlDraft.trim() || safeData.useIrohPublicNetwork }}
			>
				<PlusIcon size={14} />
				{m['pages.network.relay_settings.custom_relays.add_button']()}
			</AcerolaButton>
		</div>
		{#if customUrlError}
			<p class="text-xs text-destructive">{m['pages.network.relay_settings.invalid_url']()}</p>
		{/if}
	</div>

	<div class="space-y-2" class:opacity-50={safeData.useIrohPublicNetwork}>
		<p class="text-xs font-bold tracking-widest text-muted-foreground uppercase">
			{m['pages.network.relay_settings.iroh_relays.title']()}
		</p>

		{#each safeData.irohRelayUrls as url (url)}
			<div
				class="flex items-center justify-between gap-3 rounded-xl border border-border bg-background/50 p-3"
			>
				<span class="min-w-0 flex-1 truncate text-sm text-foreground">{url}</span>
				<AcerolaButtonIcon
					events={{ onClick: () => events.onRemoveIrohRelayUrl(url) }}
					ui={{
						variant: 'ghost',
						class: 'size-8 text-muted-foreground hover:bg-destructive/10 hover:text-destructive',
						disabled: safeData.useIrohPublicNetwork,
						'aria-label': m['pages.network.relay_settings.iroh_relays.remove']()
					}}
				>
					<Trash2Icon size={14} />
				</AcerolaButtonIcon>
			</div>
		{:else}
			<p class="text-xs text-muted-foreground">
				{m['pages.network.relay_settings.iroh_relays.empty']()}
			</p>
		{/each}

		<div class="flex items-center gap-2">
			<AcerolaInput
				state={{ value: irohUrlDraft }}
				events={{
					onValueChange: (value) => {
						irohUrlDraft = value;
						irohUrlError = false;
					}
				}}
				ui={{
					placeholder: m['pages.network.relay_settings.iroh_relays.add_placeholder'](),
					class: 'flex-1',
					disabled: safeData.useIrohPublicNetwork
				}}
			/>
			<AcerolaButton
				events={{ onClick: submitIrohUrl }}
				ui={{ size: 'sm', disabled: !irohUrlDraft.trim() || safeData.useIrohPublicNetwork }}
			>
				<PlusIcon size={14} />
				{m['pages.network.relay_settings.iroh_relays.add_button']()}
			</AcerolaButton>
		</div>
		{#if irohUrlError}
			<p class="text-xs text-destructive">{m['pages.network.relay_settings.invalid_url']()}</p>
		{/if}
	</div>
</AcerolaAccordionCard>
