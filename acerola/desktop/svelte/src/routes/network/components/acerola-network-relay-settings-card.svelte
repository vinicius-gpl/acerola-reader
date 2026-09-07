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
	import { untrack } from 'svelte';
	import { slide } from 'svelte/transition';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';
	import WifiIcon from '@lucide/svelte/icons/wifi';
	import ServerIcon from '@lucide/svelte/icons/server';
	import NetworkIcon from '@lucide/svelte/icons/network';
	import GlobeIcon from '@lucide/svelte/icons/globe';
	import CheckIcon from '@lucide/svelte/icons/check';
	import KeyRoundIcon from '@lucide/svelte/icons/key-round';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
	import AcerolaAccordionCard from '$lib/components/acerola-accordion-card/acerola-accordion-card.svelte';
	import AcerolaToggleCard from '$lib/components/acerola-toggle-card/acerola-toggle-card.svelte';
	import AcerolaInput from '$lib/components/acerola-input/acerola-input.svelte';
	import AcerolaButton from '$lib/components/acerola-button/acerola-button.svelte';
	import AcerolaButtonIcon from '$lib/components/acerola-button/acerola-button-icon.svelte';
	import { m } from '$lib/paraglide/messages';
	import { cn } from '$lib/utils/cn.utils';
	import { autoAnimateList } from '$lib/utils/auto-animate.utils';
	import { checkBadgePop } from '$lib/utils/check-badge-motion.utils';

	let { data, events }: NetworkRelaySettingsCardProps = $props();

	// Fecha por padrão — a maioria dos usuários nunca precisa mexer aqui (relay do Acerola
	// já vem ligado por padrão), então não vale ocupar espaço da tela de Rede aberto.
	let expanded = $state(false);
	let customUrlDraft = $state('');
	let customUrlError = $state(false);
	let ticketDraft = $state('');
	let ticketError = $state(false);
	let ticketSaving = $state(false);
	// Cards de "relays próprios" e "ticket Iroh" escondem sua configuração por padrão (reduz
	// texto sempre-visível) — expandem sob demanda ao clicar, num toggle independente de
	// ativar/desativar a fonte em si (ver os 3 cards no template abaixo).
	let customExpanded = $state(false);
	// Pré-expandido quando ainda não há ticket salvo — sem isso, o usuário configurando pela
	// primeira vez precisaria adivinhar que precisa clicar em algo pra ver o campo de colar o
	// ticket (só é lido na primeira renderização, igual `expanded` acima).
	let ticketExpanded = $state(untrack(() => !data?.hasIrohServicesTicket));

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

	// Subtítulo do card de relays próprios — dobra o texto de contagem no lugar de um parágrafo
	// à parte, junto da lista em si (que só aparece expandida).
	const customRelaysSubtitle = $derived(
		safeData.customRelayUrls.length === 0
			? m['pages.network.relay_settings.custom_relays.empty']()
			: m['pages.network.relay_settings.custom_relays.count']({
					count: safeData.customRelayUrls.length
				})
	);

	// Subtítulo do card de rede pública Iroh — dobra o antigo parágrafo de "desabilitado até
	// configurar um ticket" (`use_iroh_public_network_disabled_hint`) no lugar de um aviso à
	// parte, já que ele só faz sentido exatamente quando não há ticket.
	const irohSubtitle = $derived(
		safeData.hasIrohServicesTicket
			? m['pages.network.relay_settings.use_iroh_public_network_desc']()
			: m['pages.network.relay_settings.use_iroh_public_network_disabled_hint']()
	);

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

	<div class="space-y-3">
		<!-- Card 1: relay do Acerola — toggle puro, clicar na linha inteira liga/desliga. -->
		<AcerolaToggleCard
			data={{
				title: m['pages.network.relay_settings.use_acerola_relay'](),
				subtitle: m['pages.network.relay_settings.use_acerola_relay_desc']({
					url: safeData.acerolaRelayUrl
				})
			}}
			state={{ active: safeData.useAcerolaRelay }}
			events={{ onClick: () => toggleAcerolaRelay(!safeData.useAcerolaRelay) }}
			ui={{ disabled: safeData.useIrohPublicNetwork || restarting }}
		>
			{#snippet icon()}
				<ServerIcon size={18} />
			{/snippet}
		</AcerolaToggleCard>

		<!-- Card 2: relays próprios — a linha só expande/recolhe a lista, não é o que ativa a
		     fonte (ativa sozinha ao ter pelo menos 1 URL, ver `activeSourceCount`). Continua
		     clicável mesmo com a rede pública Iroh ativa (só as ações de dentro é que travam),
		     pra sempre dar pra conferir o que já está configurado. -->
		<AcerolaToggleCard
			data={{
				title: m['pages.network.relay_settings.custom_relays.title'](),
				subtitle: customRelaysSubtitle
			}}
			state={{ active: safeData.customRelayUrls.length > 0, expanded: customExpanded }}
			events={{ onClick: () => (customExpanded = !customExpanded) }}
			ui={{ class: safeData.useIrohPublicNetwork ? 'opacity-50' : undefined }}
		>
			{#snippet icon()}
				<NetworkIcon size={18} />
			{/snippet}

			{#snippet children()}
				<div class="space-y-2" use:autoAnimateList>
					{#each safeData.customRelayUrls as url (url)}
						<div
							class="flex items-center justify-between gap-3 rounded-xl border border-border bg-background/50 p-3"
						>
							<span class="min-w-0 flex-1 truncate text-sm text-foreground">{url}</span>
							<AcerolaButtonIcon
								events={{ onClick: () => removeCustomUrl(url) }}
								ui={{
									variant: 'ghost',
									class:
										'size-8 text-muted-foreground hover:bg-destructive/10 hover:text-destructive',
									disabled: safeData.useIrohPublicNetwork || restarting,
									'aria-label': m['pages.network.relay_settings.custom_relays.remove']()
								}}
							>
								<Trash2Icon size={14} />
							</AcerolaButtonIcon>
						</div>
					{/each}
				</div>

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
					<p class="text-xs text-destructive">
						{m['pages.network.relay_settings.invalid_url']()}
					</p>
				{/if}
			{/snippet}
		</AcerolaToggleCard>

		<!-- Card 3: rede pública Iroh — a linha inteira liga/desliga (exclusivo com os outros
		     dois, trava sem ticket configurado); gerenciar o ticket é uma ação à parte, dentro
		     do card, pra não confundir "ativar a fonte" com "trocar a credencial". -->
		<div
			class={cn(
				'overflow-hidden rounded-2xl border-2 transition-all duration-200',
				safeData.useIrohPublicNetwork
					? 'border-primary bg-primary/5 shadow-sm shadow-primary/10'
					: 'border-border/60 bg-card hover:border-muted-foreground/50'
			)}
		>
			<button
				type="button"
				onclick={() => toggleIrohPublicNetwork(!safeData.useIrohPublicNetwork)}
				disabled={!safeData.hasIrohServicesTicket || restarting}
				class="flex w-full items-center gap-3 p-4 text-left enabled:cursor-pointer disabled:cursor-not-allowed disabled:opacity-50"
			>
				<div
					class="flex size-10 shrink-0 items-center justify-center rounded-xl bg-muted text-foreground"
				>
					<GlobeIcon size={18} />
				</div>
				<div class="min-w-0 flex-1">
					<p class="text-sm font-semibold text-foreground">
						{m['pages.network.relay_settings.use_iroh_public_network']()}
					</p>
					<p class="truncate text-xs text-muted-foreground">{irohSubtitle}</p>
				</div>
				{#if safeData.useIrohPublicNetwork}
					<div
						use:checkBadgePop
						class="flex size-5 shrink-0 items-center justify-center rounded-full bg-primary text-primary-foreground"
					>
						<CheckIcon size={12} strokeWidth={3} />
					</div>
				{/if}
			</button>

			<div class="border-t border-border/60 p-3">
				<button
					type="button"
					onclick={() => (ticketExpanded = !ticketExpanded)}
					class="flex w-full cursor-pointer items-center gap-2 text-xs font-semibold text-muted-foreground hover:text-foreground"
				>
					<KeyRoundIcon size={12} />
					{safeData.hasIrohServicesTicket
						? m['pages.network.relay_settings.iroh_services_ticket.configured']()
						: m['pages.network.relay_settings.iroh_services_ticket.not_configured']()}
				</button>

				{#if ticketExpanded}
					<div transition:slide={{ duration: 200 }} class="mt-3 space-y-2">
						<!-- Campo em linha própria, largura cheia — dividir a linha com o botão
						     espremia o campo a ponto do placeholder (bem mais longo que qualquer
						     URL de relay) quebrar em várias linhas. -->
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
										class:
											'size-8 text-muted-foreground hover:bg-destructive/10 hover:text-destructive',
										disabled: ticketSaving,
										'aria-label':
											m['pages.network.relay_settings.iroh_services_ticket.remove_button']()
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
				{/if}
			</div>
		</div>
	</div>

	<div
		class="flex items-center justify-between gap-4 rounded-xl border border-border bg-background/50 p-3"
	>
		<div class="min-w-0">
			<p class="text-sm font-semibold text-foreground">
				{m['pages.network.relay_settings.restart.button']()}
			</p>
			{#if restartError}
				<p class="mt-1 text-xs text-destructive">
					{m['pages.network.relay_settings.restart.error']()}
				</p>
			{/if}
		</div>
		<AcerolaButton
			events={{ onClick: restart }}
			ui={{
				size: 'sm',
				variant: 'outline',
				disabled: restarting,
				title: m['pages.network.relay_settings.restart.description']()
			}}
		>
			<RefreshCwIcon size={14} class={cn(restarting && 'animate-spin')} />
			{restarting
				? m['pages.network.relay_settings.restart.restarting']()
				: m['pages.network.relay_settings.restart.button']()}
		</AcerolaButton>
	</div>
</AcerolaAccordionCard>
