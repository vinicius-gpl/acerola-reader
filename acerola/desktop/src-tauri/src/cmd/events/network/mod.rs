use std::collections::HashSet;

use acerola_p2p::api::{
    identity::DeviceInfo, network::NetworkMode, peer, transport::ACEROLA_DEFAULT_RELAY_URL,
};
use serde::{Deserialize, Serialize};

use crate::bios::scopes::RelaySettings;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfoPayload {
    pub name: String,
    pub os: String,
    pub version: String,
}

impl From<DeviceInfo> for DeviceInfoPayload {
    fn from(d: DeviceInfo) -> Self {
        Self { name: d.name, os: d.os, version: d.version }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConnectedPeerPayload {
    pub peer_id: String,
    pub alpn: String,
    pub device: Option<DeviceInfoPayload>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NetworkStatusPayload {
    pub mode: String,
    pub peers: Vec<ConnectedPeerPayload>,
}

impl NetworkStatusPayload {
    pub fn from(
        mode: NetworkMode, peers: Vec<(peer::PeerIdentity, HashSet<Vec<u8>>, Option<DeviceInfo>)>,
    ) -> Self {
        let mode_str = match mode {
            NetworkMode::Local => "local",
            NetworkMode::Relay => "relay",
        };

        let peer_list: Vec<ConnectedPeerPayload> = peers
            .into_iter()
            .flat_map(|(peer, alpns, device_info)| {
                let peer_id = peer.id;
                let device = device_info.map(DeviceInfoPayload::from);

                alpns.into_iter().map(move |alpn| ConnectedPeerPayload {
                    peer_id: peer_id.clone(),
                    alpn: String::from_utf8_lossy(&alpn).into_owned(),
                    device: device.clone(),
                })
            })
            .collect();

        Self { mode: mode_str.to_string(), peers: peer_list }
    }
}

/// Peer já pareado (TOFU) alguma vez, com o último endereço conhecido pra disparar `connect`.
/// `device_name` vem de `AcerolaP2p::known_peers()` (ver `NetworkServiceApi::paired_peers`) —
/// persiste entre reinícios, ao contrário do `DeviceInfo` de `ConnectedPeerPayload` (só existe
/// pros poucos segundos em que a sessão de handshake está de fato aberta). O frontend só cai
/// pro id cru quando esse peer nunca respondeu a entrevista de identidade.
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PairedPeerPayload {
    pub peer_id: String,
    pub addrs: Vec<u8>,
    pub device_name: Option<String>,
}

impl From<(peer::PeerAddr, Option<DeviceInfo>)> for PairedPeerPayload {
    fn from((addr, device_info): (peer::PeerAddr, Option<DeviceInfo>)) -> Self {
        Self { peer_id: addr.id.id, addrs: addr.addrs, device_name: device_info.map(|d| d.name) }
    }
}

/// Estado completo de configuração de relay, combinando as fontes lidas de
/// `settings.json` (ver [`RelaySettings`]). Trocar qualquer campo só tem efeito no
/// próximo início do app, já que a lib não suporta trocar a configuração de relay em
/// runtime.
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RelayInfo {
    /// URL fixa do relay oficial do Acerola, pra exibir mesmo quando desabilitado.
    pub acerola_relay_url: String,
    pub use_acerola_relay: bool,
    pub use_iroh_public_network: bool,
    pub custom_relay_urls: Vec<String>,
    /// Só indica SE um ticket da Iroh Services já foi colado e salvo — o valor em si nunca
    /// volta pro frontend (é uma credencial real, ver `SecureP2pStorage::load_iroh_services_ticket`).
    pub has_iroh_services_ticket: bool,
}

impl RelayInfo {
    pub fn new(settings: RelaySettings, has_iroh_services_ticket: bool) -> Self {
        Self {
            acerola_relay_url: ACEROLA_DEFAULT_RELAY_URL.to_string(),
            use_acerola_relay: settings.use_acerola_relay,
            use_iroh_public_network: settings.use_iroh_public_network,
            custom_relay_urls: settings.custom_relay_urls,
            has_iroh_services_ticket,
        }
    }
}
