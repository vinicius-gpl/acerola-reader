import { invoke } from '@tauri-apps/api/core';
import { load } from '@tauri-apps/plugin-store';
import { STORE_FILE, STORE_KEYS } from '$lib/constants/store-plugin';
import { NETWORK_COMMANDS } from '$lib/contracts/network/network.commands';
import type { RelayInfo } from '$lib/contracts/network/network.payloads';

type RelayUrlListField = 'customRelayUrls' | 'irohRelayUrls';

const LIST_STORE_KEY: Record<RelayUrlListField, string> = {
	customRelayUrls: STORE_KEYS.relayCustomUrls,
	irohRelayUrls: STORE_KEYS.relayIrohUrls
};

export function useRelaySettings() {
	let relayInfo = $state<RelayInfo | undefined>(undefined);

	async function loadRelayInfo() {
		relayInfo = await invoke<RelayInfo>(NETWORK_COMMANDS.getRelayInfo);
	}

	/// Todas as mudanças abaixo só têm efeito no próximo início do app — a conexão P2P
	/// atual continua usando o relay com o qual já subiu.

	async function setUseAcerolaRelay(value: boolean) {
		const store = await load(STORE_FILE);
		await store.set(STORE_KEYS.relayUseAcerola, value);
		await store.save();

		if (relayInfo) relayInfo = { ...relayInfo, useAcerolaRelay: value };
	}

	async function setUseIrohPublicNetwork(value: boolean) {
		const store = await load(STORE_FILE);
		await store.set(STORE_KEYS.relayUseIrohPublic, value);
		await store.save();

		if (relayInfo) relayInfo = { ...relayInfo, useIrohPublicNetwork: value };
	}

	/// Diferente das demais fontes: o ticket não vai pro `settings.json` (tauri-plugin-store),
	/// vai pro cofre criptografado do backend (mesma proteção da identidade P2P) — por isso
	/// passa pelo comando Tauri, não pelo `store` direto. O valor em si nunca volta pro
	/// frontend; só `hasIrohServicesTicket` é atualizado depois de salvar/limpar com sucesso.
	/// Propaga o erro do backend (ex: ticket malformado) pro chamador tratar (toast).
	async function setIrohServicesTicket(ticket: string) {
		await invoke(NETWORK_COMMANDS.setIrohServicesTicket, { ticket });
		if (relayInfo) relayInfo = { ...relayInfo, hasIrohServicesTicket: true };
	}

	async function clearIrohServicesTicket() {
		await invoke(NETWORK_COMMANDS.clearIrohServicesTicket);
		if (relayInfo) relayInfo = { ...relayInfo, hasIrohServicesTicket: false };
	}

	async function addUrlToList(field: RelayUrlListField, url: string) {
		const trimmed = url.trim();
		if (!trimmed || !relayInfo || relayInfo[field].includes(trimmed)) return;

		const next = [...relayInfo[field], trimmed];
		const store = await load(STORE_FILE);
		await store.set(LIST_STORE_KEY[field], next);
		await store.save();

		relayInfo = { ...relayInfo, [field]: next };
	}

	async function removeUrlFromList(field: RelayUrlListField, url: string) {
		if (!relayInfo) return;

		const next = relayInfo[field].filter((existing) => existing !== url);
		const store = await load(STORE_FILE);
		await store.set(LIST_STORE_KEY[field], next);
		await store.save();

		relayInfo = { ...relayInfo, [field]: next };
	}

	return {
		loadRelayInfo,
		setUseAcerolaRelay,
		setUseIrohPublicNetwork,
		addCustomRelayUrl: (url: string) => addUrlToList('customRelayUrls', url),
		removeCustomRelayUrl: (url: string) => removeUrlFromList('customRelayUrls', url),
		addIrohRelayUrl: (url: string) => addUrlToList('irohRelayUrls', url),
		removeIrohRelayUrl: (url: string) => removeUrlFromList('irohRelayUrls', url),
		setIrohServicesTicket,
		clearIrohServicesTicket,
		get relayInfo() {
			return relayInfo;
		},
		/// Nenhuma fonte de relay ativa — estado natural de "tudo desligado", não um modo
		/// escolhido explicitamente (espelha `RelaySettings::resolve` no backend).
		get isMdnsOnly() {
			return (
				!!relayInfo &&
				!relayInfo.useAcerolaRelay &&
				!relayInfo.useIrohPublicNetwork &&
				relayInfo.customRelayUrls.length === 0 &&
				relayInfo.irohRelayUrls.length === 0
			);
		}
	};
}
