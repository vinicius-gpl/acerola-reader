<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import { error } from '@tauri-apps/plugin-log';
	import { invoke } from '@tauri-apps/api/core';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';

	import { m } from '$lib/paraglide/messages';
	import AcerolaButtonIcon from '$lib/components/acerola-button/acerola-button-icon.svelte';
	import AcerolaAlertDialog from '$lib/components/acerola-alert-dialog/acerola-alert-dialog.svelte';
	import AcerolaRemoteLibraryDialog from '$lib/components/acerola-remote-library-dialog/acerola-remote-library-dialog.svelte';
	import AcerolaNetworkMyDeviceCard from './components/acerola-network-my-device-card.svelte';
	import AcerolaNetworkRelaySettingsCard from './components/acerola-network-relay-settings-card.svelte';
	import AcerolaNetworkPairingCard from './components/acerola-network-pairing-card.svelte';
	import AcerolaNetworkConnectCard from './components/acerola-network-connect-card.svelte';
	import AcerolaNetworkPeerList, {
		type DisplayPeer
	} from './components/acerola-network-peer-list.svelte';
	import AcerolaNetworkTransfersLog from './components/acerola-network-transfers-log.svelte';

	import { usePeerConnection } from '$lib/hooks/store/use-peer-connection.svelte';
	import { useNetworkSync } from '$lib/hooks/store/use-network-sync.svelte';
	import { useRemoteLibrary } from '$lib/hooks/store/use-remote-library.svelte';
	import { useRelaySettings } from '$lib/hooks/preferences/use-relay-settings.svelte';
	import { NETWORK_COMMANDS } from '$lib/contracts/network/network.commands';
	import { NETWORK_EVENTS } from '$lib/contracts/network/network.events';
	import { shortId } from '$lib/utils/connection-code.utils';

	import ShieldAlertIcon from '@lucide/svelte/icons/shield-alert';
	import XIcon from '@lucide/svelte/icons/x';

	const peers = usePeerConnection();
	const sync = useNetworkSync();
	const relay = useRelaySettings();
	const remoteLibrary = useRemoteLibrary();

	let browsingPeerId = $state<string | null>(null);
	let peerPendingRemoval = $state<DisplayPeer | null>(null);
	// Não é um toast: enquanto verdadeiro, a chave mestra que protege identidade/peers/confiança
	// está sem a proteção do keyring do SO (ver `security:keyring_unavailable` no backend,
	// `infra::security::MasterKeySource::FallbackFile`) — precisa ficar visível até o usuário
	// dispensar, não sumir sozinho como um toast.
	let keyringUnavailable = $state(false);
	let keyringWarningDismissed = $state(false);

	onMount(() => {
		let unlistenKeyringWarning: UnlistenFn | undefined;

		(async () => {
			// `setup_network` (backend) roda numa task assíncrona à parte e pode terminar antes
			// desta tela montar — um listener sozinho perderia o evento. Consulta o estado atual
			// sob demanda; o listener abaixo cobre o caso do backend ainda não ter terminado.
			try {
				keyringUnavailable = await invoke<boolean>(NETWORK_COMMANDS.getSecurityStatus);
			} catch (err) {
				error(`failed to query security status: ${err}`);
			}

			unlistenKeyringWarning = await listen(NETWORK_EVENTS.keyringUnavailable, () => {
				keyringUnavailable = true;
			});

			await Promise.all([
				peers.loadLocalInfo(),
				peers.startListening(),
				sync.startListening(),
				remoteLibrary.startListening(),
				relay.loadRelayInfo()
			]);
		})();

		return () => {
			unlistenKeyringWarning?.();
			peers.stopListening();
			sync.stopListening();
			remoteLibrary.stopListening();
		};
	});

	// Une os pareados persistidos (sobrevivem a restart, `deviceName` vem de `known_peers()`
	// no backend — ver `NetworkServiceApi::paired_peers`) com os conectados agora
	// (`network:status`, mais fresco mas só existe nos poucos segundos em que a conexão de
	// handshake está de fato aberta). Sem os pareados, o botão de sync só existiria no
	// instante exato em que o peer está conectado — quase nunca. `connected` é derivado da
	// presença em `status.peers`; ao sobrescrever com a entrada "ao vivo", mantém o
	// `deviceName` já persistido como fallback pro caso raro do `device` ainda não ter
	// chegado nesse instante exato (handshake acabou de abrir).
	const uniquePeers = $derived.by(() => {
		const byId = new Map<string, DisplayPeer>();
		for (const peer of peers.pairedPeers) {
			byId.set(peer.peerId, { peerId: peer.peerId, deviceName: peer.deviceName, connected: false });
		}

		for (const peer of peers.status?.peers ?? []) {
			byId.set(peer.peerId, {
				peerId: peer.peerId,
				deviceName: peer.device?.name ?? byId.get(peer.peerId)?.deviceName ?? null,
				connected: true
			});
		}
		return [...byId.values()];
	});

	function peerStatusLabel(peer: DisplayPeer): string {
		if (peer.connected) return m['pages.network.peers.online']();
		const timestamp = sync.lastSyncedAt(peer.peerId);
		if (!timestamp) return m['pages.network.peers.never_synced']();
		return m['pages.network.peers.last_synced']({ when: new Date(timestamp).toLocaleString() });
	}

	async function withSync(
		action: (peerId: string, addrs: number[]) => Promise<unknown>,
		peerId: string
	) {
		const addrs = peers.getKnownAddr(peerId);
		if (!addrs) return;

		try {
			await action(peerId, addrs);
		} catch (err) {
			toast.error(String(err));
		}
	}

	async function openRemoteLibrary(peerId: string) {
		browsingPeerId = peerId;
		await withSync((id, addrs) => remoteLibrary.queryRemoteLibrary(id, addrs), peerId);
	}

	async function handleSyncRemoteComic(peerId: string, comicName: string) {
		// Vem da navegação da biblioteca remota — o usuário só pode escolher um quadrinho que
		// ainda não tem, então é sempre pull.
		await withSync((id, addrs) => sync.syncComic(id, addrs, comicName, 'pull'), peerId);
	}
</script>

<div class="mx-auto w-full max-w-5xl space-y-12 p-8">
	<!-- Header -->
	<div>
		<h1 class="text-3xl font-bold tracking-tight text-foreground">
			{m['pages.network.title']()}
		</h1>
		<p class="mt-2 text-muted-foreground">
			{m['pages.network.subtitle']()}
		</p>
	</div>

	{#if keyringUnavailable && !keyringWarningDismissed}
		<div
			class="flex items-start gap-3 rounded-2xl border border-destructive/30 bg-destructive/5 p-4"
		>
			<ShieldAlertIcon size={18} class="mt-0.5 shrink-0 text-destructive" />
			<p class="flex-1 text-sm text-foreground">{m['pages.network.keyring_unavailable']()}</p>
			<AcerolaButtonIcon
				events={{ onClick: () => (keyringWarningDismissed = true) }}
				ui={{
					variant: 'ghost',
					class:
						'size-8 shrink-0 rounded-md p-1 text-muted-foreground hover:bg-muted hover:text-foreground',
					'aria-label': m['pages.network.dismiss']()
				}}
			>
				<XIcon size={14} />
			</AcerolaButtonIcon>
		</div>
	{/if}

	<AcerolaNetworkMyDeviceCard
		data={{
			deviceName: peers.localDeviceInfo?.name,
			localId: peers.localId,
			mode: peers.status?.mode
		}}
		events={{ onRenameDevice: (name) => peers.setDeviceName(name) }}
	/>

	<AcerolaNetworkRelaySettingsCard
		data={relay.relayInfo}
		events={{
			onToggleAcerolaRelay: (value) => relay.setUseAcerolaRelay(value),
			onToggleIrohPublicNetwork: (value) => relay.setUseIrohPublicNetwork(value),
			onAddCustomRelayUrl: (url) => relay.addCustomRelayUrl(url),
			onRemoveCustomRelayUrl: (url) => relay.removeCustomRelayUrl(url),
			onAddIrohRelayUrl: (url) => relay.addIrohRelayUrl(url),
			onRemoveIrohRelayUrl: (url) => relay.removeIrohRelayUrl(url),
			onSetIrohServicesTicket: (ticket) => relay.setIrohServicesTicket(ticket),
			onClearIrohServicesTicket: () => relay.clearIrohServicesTicket()
		}}
	/>

	<AcerolaNetworkPairingCard data={{ code: peers.connectionCode() }} />

	<AcerolaNetworkConnectCard
		data={{ connecting: peers.connecting }}
		events={{ onConnect: (code) => peers.connectWithCode(code) }}
	/>

	<AcerolaNetworkPeerList
		data={{
			peers: uniquePeers,
			addrFor: (peerId) => peers.getKnownAddr(peerId),
			statusLabel: peerStatusLabel,
			isSyncing: (peerId, kind) => sync.isSyncing(peerId, kind)
		}}
		events={{
			onSyncHistory: (peerId) => withSync((id, addrs) => sync.syncHistory(id, addrs), peerId),
			onSyncFiles: (peerId) => withSync((id, addrs) => sync.syncFiles(id, addrs), peerId),
			onSyncAll: (peerId) => withSync((id, addrs) => sync.syncAll(id, addrs), peerId),
			onBrowseLibrary: openRemoteLibrary,
			onRemove: (peer) => (peerPendingRemoval = peer)
		}}
	/>

	<!-- Transferências -->
	<section class="space-y-4">
		<div
			class="flex items-center gap-3 text-xs font-bold tracking-widest text-muted-foreground uppercase"
		>
			{m['pages.network.transfers.title']()}
		</div>
		<AcerolaNetworkTransfersLog data={{ entries: sync.log, peerLabel: peers.peerLabel }} />
	</section>
</div>

<!-- Buscar biblioteca remota -->
<AcerolaRemoteLibraryDialog
	state={{ open: browsingPeerId !== null }}
	data={{
		peerLabel: browsingPeerId ? peers.peerLabel(browsingPeerId) : '',
		comics: browsingPeerId ? remoteLibrary.comicsFor(browsingPeerId) : [],
		isLoading: browsingPeerId ? remoteLibrary.isLoading(browsingPeerId) : false,
		errorMessage: browsingPeerId ? remoteLibrary.errorFor(browsingPeerId) : undefined,
		coverPathFor: (comicName) =>
			browsingPeerId ? remoteLibrary.coverPathFor(browsingPeerId, comicName) : undefined,
		isSyncing: () => (browsingPeerId ? sync.isSyncing(browsingPeerId, 'comic') : false)
	}}
	events={{
		onOpenChange: (open) => {
			if (!open) browsingPeerId = null;
		},
		onSelectComic: (comicName) => {
			if (browsingPeerId) handleSyncRemoteComic(browsingPeerId, comicName);
		}
	}}
/>

<!-- Confirmar remoção de dispositivo pareado -->
<AcerolaAlertDialog
	state={{ open: peerPendingRemoval !== null }}
	data={{
		title: m['pages.network.peers.remove_confirm.title']({
			peer: peerPendingRemoval
				? (peerPendingRemoval.deviceName ?? shortId(peerPendingRemoval.peerId))
				: ''
		}),
		description: m['pages.network.peers.remove_confirm.desc'](),
		cancelText: m['pages.network.peers.remove_confirm.cancel'](),
		actionText: m['pages.network.peers.remove_confirm.confirm']()
	}}
	ui={{ variant: 'destructive' }}
	events={{
		onOpenChange: (open) => {
			if (!open) peerPendingRemoval = null;
		},
		onCancel: () => (peerPendingRemoval = null),
		onAction: () => {
			const peerId = peerPendingRemoval?.peerId;
			peerPendingRemoval = null;
			if (peerId) peers.removePeer(peerId);
		}
	}}
/>
