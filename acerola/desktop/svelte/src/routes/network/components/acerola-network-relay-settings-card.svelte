<script module lang="ts">
	export type NetworkRelaySettingsCardData = {
		acerolaRelayUrl: string;
		useAcerolaRelay: boolean;
		useIrohPublicNetwork: boolean;
		customRelayUrls: string[];
		hasIrohServicesTicket: boolean;
	};

	export type NetworkRelaySettingsCardProps = {
		data: NetworkRelaySettingsCardData | undefined;
		events: {
			onToggleAcerolaRelay: (value: boolean) => Promise<void>;
			onToggleIrohPublicNetwork: (value: boolean) => Promise<void>;
			onAddCustomRelayUrl: (url: string) => Promise<void>;
			onRemoveCustomRelayUrl: (url: string) => Promise<void>;
			onSetIrohServicesTicket: (ticket: string) => Promise<void>;
			onClearIrohServicesTicket: () => Promise<void>;
			onRestart: () => Promise<void>;
		};
	};
</script>

<script lang="ts">
	import PlusIcon from '@lucide/svelte/icons/plus';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';
	import WifiIcon from '@lucide/svelte/icons/wifi';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
	import AcerolaAccordionCard from '$lib/components/acerola-accordion-card/acerola-accordion-card.svelte';
	import AcerolaSwitch from '$lib/components/acerola-switch/acerola-switch.svelte';
	import AcerolaInput from '$lib/components/acerola-input/acerola-input.svelte';
	import AcerolaButton from '$lib/components/acerola-button/acerola-button.svelte';
	import AcerolaButtonIcon from '$lib/components/acerola-button/acerola-button-icon.svelte';
	import { m } from '$lib/paraglide/messages';
	import { cn } from '$lib/utils/cn.utils';

	let { data, events }: NetworkRelaySettingsCardProps = $props();

	// Fecha por padrão — a maioria dos usuários nunca precisa mexer aqui (relay do Acerola
	// já vem ligado por padrão), então não vale ocupar espaço da tela de Rede aberto.
	let expanded = $state(false);
	let customUrlDraft = $state('');
	let customUrlError = $state(false);
	let ticketDraft = $state('');
	let ticketError = $state(false);
	let ticketSaving = $state(false);

	// Compartilhado por TODA ação que reinicia o node P2P por baixo (toggle de relay,
	// add/remove de URL própria, botão manual) — as duas coisas que faltavam antes: (1)
	// feedback visual de que uma restart está em andamento (os toggles não tinham NENHUM,
	// diferente do botão manual, que já tinha `restarting`) e (2) trava contra reentrância —
	// clique duplo no mesmo controle, ou mexer em outro enquanto uma restart anterior ainda
	// não terminou, agora é ignorado em vez de disparar uma segunda reconstrução do node em
	// paralelo (a causa raiz real: duas reconstruções concorrentes registravam a MESMA
	// identidade no relay ao mesmo tempo, e o relay derrubava uma delas em silêncio).
	let restarting = $state(false);
	let restartError = $state(false);

	async function runRestartingAction(action: () => Promise<void>) {
		if (restarting) return;
		restarting = true;
		restartError = false;
		try {
			await action();
		} catch {
			restartError = true;
		} finally {
			restarting = false;
		}
	}

	let safeData = $derived(
		data ?? {
			acerolaRelayUrl: '',
			useAcerolaRelay: false,
			useIrohPublicNetwork: false,
			customRelayUrls: [],
			hasIrohServicesTicket: false
		}
	);

	const activeSourceCount = $derived(
		(safeData.useAcerolaRelay ? 1 : 0) + safeData.customRelayUrls.length
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

	function toggleAcerolaRelay(value: boolean) {
		runRestartingAction(() => events.onToggleAcerolaRelay(value));
	}

	function toggleIrohPublicNetwork(value: boolean) {
		runRestartingAction(() => events.onToggleIrohPublicNetwork(value));
	}

	function submitCustomUrl() {
		const trimmed = customUrlDraft.trim();
		if (!trimmed) return;
		if (!isValidUrl(trimmed)) {
			customUrlError = true;
			return;
		}
		customUrlError = false;
		customUrlDraft = '';
		runRestartingAction(() => events.onAddCustomRelayUrl(trimmed));
	}

	function removeCustomUrl(url: string) {
		runRestartingAction(() => events.onRemoveCustomRelayUrl(url));
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

	function restart() {
		runRestartingAction(() => events.onRestart());
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
			events={{ onCheckedChange: toggleAcerolaRelay }}
			ui={{ disabled: safeData.useIrohPublicNetwork || restarting }}
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
			events={{ onCheckedChange: toggleIrohPublicNetwork }}
			ui={{ disabled: !safeData.hasIrohServicesTicket || restarting }}
		/>
	</div>

	<!-- Sem isso, o switch cinza acima não explica por conta própria por que está travado — o
	     usuário precisa saber que a solução é colar um ticket na seção logo abaixo. -->
	{#if !safeData.hasIrohServicesTicket}
		<p class="text-xs text-muted-foreground italic">
			{m['pages.network.relay_settings.use_iroh_public_network_disabled_hint']()}
		</p>
	{/if}

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

		<!-- Campo em linha própria, largura cheia — dividir a linha com o botão espremia o
		     campo a ponto do placeholder (bem mais longo que qualquer URL de relay) quebrar
		     em várias linhas. -->
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
				class: 'w-full',
				disabled: ticketSaving
			}}
		/>

		{#if ticketError}
			<p class="text-xs text-destructive">
				{m['pages.network.relay_settings.iroh_services_ticket.invalid']()}
			</p>
		{/if}

		<div class="flex items-center justify-end gap-2">
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
			<AcerolaButton
				events={{ onClick: submitTicket }}
				ui={{ size: 'sm', disabled: !ticketDraft.trim() || ticketSaving }}
			>
				{safeData.hasIrohServicesTicket
					? m['pages.network.relay_settings.iroh_services_ticket.replace_button']()
					: m['pages.network.relay_settings.iroh_services_ticket.save_button']()}
			</AcerolaButton>
		</div>

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
					events={{ onClick: () => removeCustomUrl(url) }}
					ui={{
						variant: 'ghost',
						class: 'size-8 text-muted-foreground hover:bg-destructive/10 hover:text-destructive',
						disabled: safeData.useIrohPublicNetwork || restarting,
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
					disabled: safeData.useIrohPublicNetwork || restarting
				}}
			/>
			<AcerolaButton
				events={{ onClick: submitCustomUrl }}
				ui={{
					size: 'sm',
					disabled: !customUrlDraft.trim() || safeData.useIrohPublicNetwork || restarting
				}}
			>
				<PlusIcon size={14} />
				{m['pages.network.relay_settings.custom_relays.add_button']()}
			</AcerolaButton>
		</div>
		{#if customUrlError}
			<p class="text-xs text-destructive">{m['pages.network.relay_settings.invalid_url']()}</p>
		{/if}
	</div>

	<div
		class="flex items-center justify-between gap-4 rounded-xl border border-border bg-background/50 p-3"
	>
		<div class="min-w-0">
			<p class="text-sm font-semibold text-foreground">
				{m['pages.network.relay_settings.restart.button']()}
			</p>
			<p class="text-xs text-muted-foreground">
				{m['pages.network.relay_settings.restart.description']()}
			</p>
			{#if restartError}
				<p class="mt-1 text-xs text-destructive">
					{m['pages.network.relay_settings.restart.error']()}
				</p>
			{/if}
		</div>
		<AcerolaButton
			events={{ onClick: restart }}
			ui={{ size: 'sm', variant: 'outline', disabled: restarting }}
		>
			<RefreshCwIcon size={14} class={cn(restarting && 'animate-spin')} />
			{restarting
				? m['pages.network.relay_settings.restart.restarting']()
				: m['pages.network.relay_settings.restart.button']()}
		</AcerolaButton>
	</div>
</AcerolaAccordionCard>
