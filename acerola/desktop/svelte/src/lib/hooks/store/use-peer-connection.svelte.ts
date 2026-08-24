import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { NETWORK_COMMANDS } from '$lib/contracts/network/network.commands';
import { NETWORK_EVENTS } from '$lib/contracts/network/network.events';
import type {
	DeviceInfoPayload,
	NetworkStatusPayload,
	PairedPeerPayload,
	RelayInfo
} from '$lib/contracts/network/network.payloads';
import {
	decodeConnectionCode,
	encodeConnectionCode,
	InvalidConnectionCodeError,
	shortId,
	type LocalPeerAddr
} from '$lib/utils/connection-code.utils';

export { InvalidConnectionCodeError, shortId } from '$lib/utils/connection-code.utils';

const HANDSHAKE_ALPN = 'acerola/handshake/1';

/// Estado de pareamento: identidade local (pro QR/código), status da rede (peers
/// conectados via evento `network:status`) e a ação de conectar em um peer colando um
/// código gerado por esse mesmo hook em outro dispositivo.
export function usePeerConnection() {
	let localId = $state<string | undefined>(undefined);
	let localAddr = $state<LocalPeerAddr | undefined>(undefined);
	let localDeviceInfo = $state<DeviceInfoPayload | undefined>(undefined);
	let relayInfo = $state<RelayInfo | undefined>(undefined);
	let status = $state<NetworkStatusPayload | undefined>(undefined);
	let pairedPeers = $state<PairedPeerPayload[]>([]);
	let connecting = $state(false);

	// `network:status` só traz peerId/alpn/device — não os bytes de endereço, que a lib
	// exige de novo pra abrir uma conexão em outro ALPN (ex: disparar um sync depois do
	// pareamento inicial). Guardamos aqui os `addrs` de cada peer conhecido — populado tanto
	// ao parear via código colado quanto (principalmente) por `loadPairedPeers()`, que é o
	// que faz o botão de sync existir pra um peer que não está conectado agora (a conexão do
	// protocolo em si dura só segundos — ver `NetworkServiceApi::paired_peers`).
	const knownAddrs = new Map<string, number[]>();

	let unlistenStatus: UnlistenFn | undefined;
	let unlistenDeviceInfoReceived: UnlistenFn | undefined;
	let unlistenDeviceInfoExchanged: UnlistenFn | undefined;

	// `startListening`/`loadPairedPeers`/etc são async e podem seguir rodando depois que
	// `stopListening` já rodou (unmount no meio de um await) — sem essa flag, a continuação
	// tenta gravar em $state de uma instância já descartada (`usePeerConnection()` é sempre
	// uma instância nova por mount, então não precisa resetar em `startListening`).
	let disposed = false;

	async function loadLocalInfo() {
		const result = await Promise.all([
			invoke<string>(NETWORK_COMMANDS.getLocalId),
			invoke<LocalPeerAddr>(NETWORK_COMMANDS.getLocalAddr),
			invoke<DeviceInfoPayload>(NETWORK_COMMANDS.getLocalDeviceInfo),
			invoke<RelayInfo>(NETWORK_COMMANDS.getRelayInfo)
		]);
		if (disposed) return;
		[localId, localAddr, localDeviceInfo, relayInfo] = result;
	}

	async function refreshStatus() {
		await invoke(NETWORK_COMMANDS.getNetworkStatus);
	}

	/// Carrega a lista de peers já pareados (persistida, independe de conexão ativa) e
	/// alimenta `knownAddrs` com os endereços de cada um — sem isso, os botões de sync ficam
	/// desabilitados pra qualquer peer que não esteja conectado neste exato instante.
	async function loadPairedPeers() {
		const result = await invoke<PairedPeerPayload[]>(NETWORK_COMMANDS.getPairedPeers);
		if (disposed) return;
		pairedPeers = result;
		for (const peer of pairedPeers) {
			knownAddrs.set(peer.peerId, peer.addrs);
		}
	}

	/// Handshake concluído (de qualquer lado, ver `NETWORK_EVENTS.deviceInfoReceived` /
	/// `.deviceInfoExchanged`) — só re-consulta o status atual em vez de tentar montar o
	/// peer a partir do payload do evento (que é só o `peerId` cru, sem `device`/`alpn`).
	async function handlePeerHandshake() {
		await Promise.all([refreshStatus(), loadPairedPeers()]);
	}

	async function startListening() {
		unlistenStatus = await listen<NetworkStatusPayload>(NETWORK_EVENTS.status, (event) => {
			status = event.payload;
		});
		unlistenDeviceInfoReceived = await listen(
			NETWORK_EVENTS.deviceInfoReceived,
			handlePeerHandshake
		);
		unlistenDeviceInfoExchanged = await listen(
			NETWORK_EVENTS.deviceInfoExchanged,
			handlePeerHandshake
		);
		await Promise.all([refreshStatus(), loadPairedPeers()]);
	}

	function stopListening() {
		disposed = true;
		unlistenStatus?.();
		unlistenStatus = undefined;
		unlistenDeviceInfoReceived?.();
		unlistenDeviceInfoReceived = undefined;
		unlistenDeviceInfoExchanged?.();
		unlistenDeviceInfoExchanged = undefined;
	}

	/// O código/QR que o outro dispositivo precisa escanear ou colar pra nos alcançar.
	function connectionCode(): string | undefined {
		if (!localAddr) return undefined;
		return encodeConnectionCode(localAddr);
	}

	async function connectWithCode(code: string) {
		const parsed = decodeConnectionCode(code);

		connecting = true;
		try {
			await invoke(NETWORK_COMMANDS.connectToPeer, {
				peerId: parsed.id.id,
				addrs: parsed.addrs,
				alpn: HANDSHAKE_ALPN
			});
			knownAddrs.set(parsed.id.id, parsed.addrs);
			// O handshake já persiste esse peer no backend (`save_peer_if_present`) — recarrega
			// pra ele aparecer na lista de pareados sem esperar o próximo restart do app.
			await Promise.all([refreshStatus(), loadPairedPeers()]);
		} finally {
			connecting = false;
		}
	}

	function getKnownAddr(peerId: string): number[] | undefined {
		return knownAddrs.get(peerId);
	}

	/// Nome amigável de um peer — prioriza `network:status` (mais fresco, cobre o caso raro de
	/// um dispositivo renomeado no meio da sessão), mas cai pra `pairedPeers` (persistente,
	/// sobrevive ao handshake fechar — é o caso comum, já que a sessão de handshake em si dura
	/// só segundos) antes de cair pro id truncado.
	function peerLabel(peerId: string): string {
		const liveDevice = status?.peers.find((peer) => peer.peerId === peerId)?.device;
		if (liveDevice) return liveDevice.name;
		const paired = pairedPeers.find((peer) => peer.peerId === peerId);
		return paired?.deviceName ?? shortId(peerId);
	}

	return {
		loadLocalInfo,
		startListening,
		stopListening,
		refreshStatus,
		connectionCode,
		connectWithCode,
		getKnownAddr,
		peerLabel,
		get localId() {
			return localId;
		},
		get localDeviceInfo() {
			return localDeviceInfo;
		},
		get relayInfo() {
			return relayInfo;
		},
		get status() {
			return status;
		},
		get pairedPeers() {
			return pairedPeers;
		},
		get connecting() {
			return connecting;
		}
	};
}
