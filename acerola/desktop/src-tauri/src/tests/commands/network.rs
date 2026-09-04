use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use acerola_p2p::api::{
    identity::DeviceInfo,
    network::NetworkMode,
    peer::{PeerAddr, PeerIdentity},
};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

use super::support::{
    build_webview, invoke_err, invoke_ok, invoke_ok_value, listen_event, recv_event,
};
use crate::{
    cmd::features::network as network_cmd,
    core::services::network::{ConnectedPeerInfo, NetworkServiceApi},
};

#[derive(Clone)]
struct MockNetworkService {
    state: Arc<Mutex<MockNetworkState>>,
}

struct MockNetworkState {
    local_id: String,
    device_name: String,
    mode: NetworkMode,
    peers: Vec<ConnectedPeerInfo>,
    paired_peers: Vec<(PeerAddr, Option<DeviceInfo>)>,
    last_connection: Option<(String, Vec<u8>)>,
    failure: Option<String>,
}

impl MockNetworkService {
    fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(MockNetworkState {
                local_id: "local-peer-id".to_string(),
                device_name: "mock-device".to_string(),
                mode: NetworkMode::Local,
                peers: Vec::new(),
                paired_peers: Vec::new(),
                last_connection: None,
                failure: None,
            })),
        }
    }

    fn fail_next(&self, message: &str) {
        self.state.lock().expect("network mock mutex should not be poisoned").failure =
            Some(message.to_string());
    }

    fn set_mode(&self, mode: NetworkMode) {
        self.state.lock().expect("network mock mutex should not be poisoned").mode = mode;
    }

    fn set_peers(&self, peers: Vec<ConnectedPeerInfo>) {
        self.state.lock().expect("network mock mutex should not be poisoned").peers = peers;
    }

    fn set_paired_peers(&self, paired_peers: Vec<(PeerAddr, Option<DeviceInfo>)>) {
        self.state.lock().expect("network mock mutex should not be poisoned").paired_peers =
            paired_peers;
    }

    fn mode(&self) -> NetworkMode {
        self.state.lock().expect("network mock mutex should not be poisoned").mode.clone()
    }

    fn last_connection(&self) -> Option<(String, Vec<u8>)> {
        self.state
            .lock()
            .expect("network mock mutex should not be poisoned")
            .last_connection
            .clone()
    }

    fn take_failure(state: &mut MockNetworkState) -> Result<(), String> {
        if let Some(message) = state.failure.take() {
            Err(message)
        } else {
            Ok(())
        }
    }
}

#[async_trait]
impl NetworkServiceApi for MockNetworkService {
    fn local_id(&self) -> Result<String, String> {
        let mut state = self.state.lock().expect("network mock mutex should not be poisoned");
        Self::take_failure(&mut state)?;
        Ok(state.local_id.clone())
    }

    fn local_addr(&self) -> Result<PeerAddr, String> {
        let mut state = self.state.lock().expect("network mock mutex should not be poisoned");
        Self::take_failure(&mut state)?;
        Ok(PeerAddr {
            id: PeerIdentity { id: state.local_id.clone(), device_id: None },
            addrs: vec![],
        })
    }

    async fn local_device_info(&self) -> Result<DeviceInfo, String> {
        let mut state = self.state.lock().expect("network mock mutex should not be poisoned");
        Self::take_failure(&mut state)?;
        Ok(DeviceInfo {
            name: state.device_name.clone(),
            os: "test-os".to_string(),
            version: "0.0.0".to_string(),
        })
    }

    async fn set_local_device_name(&self, name: String) -> Result<(), String> {
        let mut state = self.state.lock().expect("network mock mutex should not be poisoned");
        Self::take_failure(&mut state)?;
        state.device_name = name;
        Ok(())
    }

    async fn connected_peers_with_info(&self) -> Result<Vec<ConnectedPeerInfo>, String> {
        let mut state = self.state.lock().expect("network mock mutex should not be poisoned");
        Self::take_failure(&mut state)?;
        Ok(state.peers.clone())
    }

    async fn paired_peers(&self) -> Result<Vec<(PeerAddr, Option<DeviceInfo>)>, String> {
        let mut state = self.state.lock().expect("network mock mutex should not be poisoned");
        Self::take_failure(&mut state)?;
        Ok(state.paired_peers.clone())
    }

    async fn remove_peer(&self, id: String) -> Result<(), String> {
        let mut state = self.state.lock().expect("network mock mutex should not be poisoned");
        Self::take_failure(&mut state)?;
        state.paired_peers.retain(|(addr, _)| addr.id.id != id);
        Ok(())
    }

    async fn switch_to_local(&self) -> Result<(), String> {
        let mut state = self.state.lock().expect("network mock mutex should not be poisoned");
        Self::take_failure(&mut state)?;
        state.mode = NetworkMode::Local;
        Ok(())
    }

    async fn switch_to_relay(&self) -> Result<(), String> {
        let mut state = self.state.lock().expect("network mock mutex should not be poisoned");
        Self::take_failure(&mut state)?;
        state.mode = NetworkMode::Relay;
        Ok(())
    }

    async fn mode(&self) -> Result<NetworkMode, String> {
        let mut state = self.state.lock().expect("network mock mutex should not be poisoned");
        Self::take_failure(&mut state)?;
        Ok(state.mode.clone())
    }

    async fn connect(&self, peer_addr: PeerAddr, alpn: Vec<u8>) -> Result<(), String> {
        let mut state = self.state.lock().expect("network mock mutex should not be poisoned");
        Self::take_failure(&mut state)?;
        state.last_connection = Some((peer_addr.id.id, alpn));
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), String> {
        let mut state = self.state.lock().expect("network mock mutex should not be poisoned");
        Self::take_failure(&mut state)
    }
}

fn mock_network_service() -> Arc<MockNetworkService> {
    Arc::new(MockNetworkService::new())
}

fn build_network_app(
    service: Arc<MockNetworkService>,
) -> Result<(tauri::App<tauri::test::MockRuntime>, tauri::WebviewWindow<tauri::test::MockRuntime>)>
{
    let managed: Arc<dyn NetworkServiceApi> = service;

    build_webview(tauri::test::mock_builder().manage(managed).invoke_handler(
        tauri::generate_handler![
            network_cmd::get_network_status,
            network_cmd::switch_to_local,
            network_cmd::switch_to_relay,
            network_cmd::get_local_id,
            network_cmd::get_local_device_info,
            network_cmd::set_local_device_name,
            network_cmd::connect_to_peer,
            network_cmd::get_paired_peers,
            network_cmd::remove_paired_peer,
        ],
    ))
}

fn peer(peer_id: &str, alpn: &[u8]) -> ConnectedPeerInfo {
    let mut alpns = HashSet::new();
    alpns.insert(alpn.to_vec());

    (
        PeerIdentity { id: peer_id.to_string(), device_id: None },
        alpns,
        Some(DeviceInfo {
            name: "Notebook".to_string(),
            os: "windows".to_string(),
            version: "0.0.1-test".to_string(),
        }),
    )
}

#[tokio::test(flavor = "multi_thread")]
async fn test_get_network_status_emits_mode_and_peers() -> Result<()> {
    let service = mock_network_service();
    service.set_mode(NetworkMode::Relay);
    service.set_peers(vec![peer("peer-1", b"acerola/handshake/1")]);
    let (app, webview) = build_network_app(service)?;
    let status_rx = listen_event(&app, "network:status");

    let _: Value = invoke_ok(&webview, "get_network_status", json!({}))?;

    let status = recv_event(status_rx, "network:status").await?;

    assert_eq!(status["mode"], "relay");
    assert_eq!(status["peers"][0]["peerId"], "peer-1");
    assert_eq!(status["peers"][0]["alpn"], "acerola/handshake/1");
    assert_eq!(status["peers"][0]["device"]["name"], "Notebook");

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_get_network_status_serializes_service_error() -> Result<()> {
    let service = mock_network_service();
    service.fail_next("status failure");
    let (_app, webview) = build_network_app(service)?;

    let error = invoke_err(&webview, "get_network_status", json!({}))?;

    assert_eq!(error, json!("status failure"));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_switch_to_local_changes_mode() -> Result<()> {
    let service = mock_network_service();
    service.set_mode(NetworkMode::Relay);
    let (_app, webview) = build_network_app(service.clone())?;

    let _: Value = invoke_ok(&webview, "switch_to_local", json!({}))?;

    assert!(matches!(service.mode(), NetworkMode::Local));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_switch_to_local_serializes_service_error() -> Result<()> {
    let service = mock_network_service();
    service.fail_next("local failure");
    let (_app, webview) = build_network_app(service)?;

    let error = invoke_err(&webview, "switch_to_local", json!({}))?;

    assert_eq!(error, json!("local failure"));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_switch_to_relay_changes_mode() -> Result<()> {
    let service = mock_network_service();
    let (_app, webview) = build_network_app(service.clone())?;

    let _: Value = invoke_ok(&webview, "switch_to_relay", json!({}))?;

    assert!(matches!(service.mode(), NetworkMode::Relay));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_switch_to_relay_serializes_service_error() -> Result<()> {
    let service = mock_network_service();
    service.fail_next("relay failure");
    let (_app, webview) = build_network_app(service)?;

    let error = invoke_err(&webview, "switch_to_relay", json!({}))?;

    assert_eq!(error, json!("relay failure"));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_get_local_id_returns_local_id() -> Result<()> {
    let service = mock_network_service();
    let (_app, webview) = build_network_app(service)?;

    let local_id = invoke_ok_value(&webview, "get_local_id", json!({}))?;

    assert_eq!(local_id, json!("local-peer-id"));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_get_local_id_serializes_service_error() -> Result<()> {
    let service = mock_network_service();
    service.fail_next("id failure");
    let (_app, webview) = build_network_app(service)?;

    let error = invoke_err(&webview, "get_local_id", json!({}))?;

    assert_eq!(error, json!("id failure"));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_set_local_device_name_updates_local_device_info() -> Result<()> {
    let service = mock_network_service();
    let (_app, webview) = build_network_app(service)?;

    let _: Value =
        invoke_ok(&webview, "set_local_device_name", json!({ "name": "Notebook do Vinicius" }))?;

    let device_info: Value = invoke_ok(&webview, "get_local_device_info", json!({}))?;
    assert_eq!(device_info["name"], "Notebook do Vinicius");

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_set_local_device_name_rejects_blank_name() -> Result<()> {
    let service = mock_network_service();
    let (_app, webview) = build_network_app(service)?;

    let error = invoke_err(&webview, "set_local_device_name", json!({ "name": "   " }))?;

    assert_eq!(error, json!("device name cannot be empty"));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_set_local_device_name_serializes_service_error() -> Result<()> {
    let service = mock_network_service();
    service.fail_next("rename failure");
    let (_app, webview) = build_network_app(service)?;

    let error = invoke_err(&webview, "set_local_device_name", json!({ "name": "New Name" }))?;

    assert_eq!(error, json!("rename failure"));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_get_paired_peers_returns_persisted_peers_regardless_of_active_connection(
) -> Result<()> {
    let service = mock_network_service();
    service.set_paired_peers(vec![
        (
            PeerAddr {
                id: PeerIdentity { id: "paired-offline-peer".to_string(), device_id: None },
                addrs: vec![9, 9, 9],
            },
            Some(DeviceInfo {
                name: "Notebook do Vinicius".to_string(),
                os: "linux".to_string(),
                version: "0.0.1".to_string(),
            }),
        ),
        // Peer pareado que nunca respondeu a entrevista de identidade (handshake antigo,
        // versão anterior a `DeviceInfo` existir, etc.) — frontend deve cair pro id cru.
        (
            PeerAddr {
                id: PeerIdentity { id: "peer-without-device-info".to_string(), device_id: None },
                addrs: vec![1, 2, 3],
            },
            None,
        ),
    ]);
    // Nenhum peer conectado agora (`set_peers` não chamado) — a lista de pareados não
    // depende disso, ao contrário de `get_network_status`.
    let (_app, webview) = build_network_app(service)?;

    let paired: Value = invoke_ok(&webview, "get_paired_peers", json!({}))?;

    assert_eq!(paired[0]["peerId"], "paired-offline-peer");
    assert_eq!(paired[0]["addrs"], json!([9, 9, 9]));
    assert_eq!(paired[0]["deviceName"], "Notebook do Vinicius");

    assert_eq!(paired[1]["peerId"], "peer-without-device-info");
    assert_eq!(paired[1]["deviceName"], Value::Null);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_get_paired_peers_serializes_service_error() -> Result<()> {
    let service = mock_network_service();
    service.fail_next("paired peers failure");
    let (_app, webview) = build_network_app(service)?;

    let error = invoke_err(&webview, "get_paired_peers", json!({}))?;

    assert_eq!(error, json!("paired peers failure"));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_remove_paired_peer_drops_it_from_the_paired_list() -> Result<()> {
    let service = mock_network_service();
    service.set_paired_peers(vec![
        (
            PeerAddr {
                id: PeerIdentity { id: "peer-to-remove".to_string(), device_id: None },
                addrs: vec![1],
            },
            None,
        ),
        (
            PeerAddr {
                id: PeerIdentity { id: "peer-to-keep".to_string(), device_id: None },
                addrs: vec![2],
            },
            None,
        ),
    ]);
    let (_app, webview) = build_network_app(service)?;

    let _: Value =
        invoke_ok(&webview, "remove_paired_peer", json!({ "peerId": "peer-to-remove" }))?;

    let paired: Value = invoke_ok(&webview, "get_paired_peers", json!({}))?;
    assert_eq!(paired.as_array().unwrap().len(), 1);
    assert_eq!(paired[0]["peerId"], "peer-to-keep");

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_remove_paired_peer_serializes_service_error() -> Result<()> {
    let service = mock_network_service();
    service.fail_next("remove failure");
    let (_app, webview) = build_network_app(service)?;

    let error = invoke_err(&webview, "remove_paired_peer", json!({ "peerId": "peer-1" }))?;

    assert_eq!(error, json!("remove failure"));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_connect_to_peer_forwards_peer_id_and_alpn_bytes() -> Result<()> {
    let service = mock_network_service();
    let (_app, webview) = build_network_app(service.clone())?;

    let _: Value = invoke_ok(
        &webview,
        "connect_to_peer",
        json!({ "peerId": "peer-2", "addrs": [], "alpn": "acerola/handshake/1" }),
    )?;

    assert_eq!(
        service.last_connection(),
        Some(("peer-2".to_string(), b"acerola/handshake/1".to_vec()))
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_connect_to_peer_serializes_service_error() -> Result<()> {
    let service = mock_network_service();
    service.fail_next("connect failure");
    let (_app, webview) = build_network_app(service)?;

    let error = invoke_err(
        &webview,
        "connect_to_peer",
        json!({ "peerId": "peer-2", "addrs": [], "alpn": "acerola/handshake/1" }),
    )?;

    assert_eq!(error, json!("connect failure"));

    Ok(())
}
