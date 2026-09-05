import { invoke } from '@tauri-apps/api/core';
import { load } from '@tauri-apps/plugin-store';
import { STORE_FILE, STORE_KEYS } from '$lib/constants/store-plugin';
import { NETWORK_COMMANDS } from '$lib/contracts/network/network.commands';
import type { RelayInfo } from '$lib/contracts/network/network.payloads';

export function useRelaySettings() {
	let relayInfo = $state<RelayInfo | undefined>(undefined);

	async function loadRelayInfo() {
		relayInfo = await invoke<RelayInfo>(NETWORK_COMMANDS.getRelayInfo);
	}

	/// Todas as mudanças abaixo persistem em `settings.json` E aplicam ao node P2P já vivo na
	/// hora (`apply_relay_settings`, relê o disco/cofre e reconfigura o `Endpoint` via
	/// `insert_relay`/`remove_relay`) — não precisa mais fechar/reabrir o app pra valer.

	async function setUseAcerolaRelay(value: boolean) {
		const store = await load(STORE_FILE);
		await store.set(STORE_KEYS.relayUseAcerola, value);
		await store.save();

		if (relayInfo) relayInfo = { ...relayInfo, useAcerolaRelay: value };
		await invoke(NETWORK_COMMANDS.applyRelaySettings);
	}

	async function setUseIrohPublicNetwork(value: boolean) {
		const store = await load(STORE_FILE);
		await store.set(STORE_KEYS.relayUseIrohPublic, value);
		await store.save();

		if (relayInfo) relayInfo = { ...relayInfo, useIrohPublicNetwork: value };
		await invoke(NETWORK_COMMANDS.applyRelaySettings);
	}

	/// Diferente das demais fontes: o ticket não vai pro `settings.json` (tauri-plugin-store),
	/// vai pro cofre criptografado do backend (mesma proteção da identidade P2P) — por isso
	/// passa pelo comando Tauri, não pelo `store` direto. O valor em si nunca volta pro
	/// frontend; só `hasIrohServicesTicket` é atualizado depois de salvar/limpar com sucesso.
	/// Propaga o erro do backend (ex: ticket malformado) pro chamador tratar (toast).
	async function setIrohServicesTicket(ticket: string) {
		await invoke(NETWORK_COMMANDS.setIrohServicesTicket, { ticket });
		if (relayInfo) relayInfo = { ...relayInfo, hasIrohServicesTicket: true };
		await invoke(NETWORK_COMMANDS.applyRelaySettings);
	}

	async function clearIrohServicesTicket() {
		await invoke(NETWORK_COMMANDS.clearIrohServicesTicket);
		if (relayInfo) relayInfo = { ...relayInfo, hasIrohServicesTicket: false };
		await invoke(NETWORK_COMMANDS.applyRelaySettings);
	}

	async function addCustomRelayUrl(url: string) {
		const trimmed = url.trim();
		if (!trimmed || !relayInfo || relayInfo.customRelayUrls.includes(trimmed)) return;

		const next = [...relayInfo.customRelayUrls, trimmed];
		const store = await load(STORE_FILE);
		await store.set(STORE_KEYS.relayCustomUrls, next);
		await store.save();

		relayInfo = { ...relayInfo, customRelayUrls: next };
		await invoke(NETWORK_COMMANDS.applyRelaySettings);
	}

	async function removeCustomRelayUrl(url: string) {
		if (!relayInfo) return;

		const next = relayInfo.customRelayUrls.filter((existing) => existing !== url);
		const store = await load(STORE_FILE);
		await store.set(STORE_KEYS.relayCustomUrls, next);
		await store.save();

		relayInfo = { ...relayInfo, customRelayUrls: next };
		await invoke(NETWORK_COMMANDS.applyRelaySettings);
	}

	return {
		loadRelayInfo,
		setUseAcerolaRelay,
		setUseIrohPublicNetwork,
		addCustomRelayUrl,
		removeCustomRelayUrl,
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
				relayInfo.customRelayUrls.length === 0
			);
		}
	};
}
