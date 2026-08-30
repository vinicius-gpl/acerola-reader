use std::sync::Arc;

use tokio::{
    sync::RwLock,
    time::{sleep, Duration},
};

use crate::{
    api::identity::DeviceInfo,
    core::network::state::NetworkState,
    data::protocol::{
        rpc::{RpcClientHandler, RpcServerHandler},
        DeviceInfoStore, EventEmitter, ProtocolHandler,
    },
    infra::peer::PeerId,
};

#[allow(dead_code)]
fn make_peer(id: &str) -> PeerId {
    PeerId { id: id.into(), device_id: None }
}

#[allow(dead_code)]
fn make_device(name: &str) -> DeviceInfo {
    DeviceInfo { name: name.into(), os: "linux".into(), version: "1.0".into() }
}

#[allow(dead_code)]
fn shared_device(name: &str) -> Arc<RwLock<DeviceInfo>> {
    Arc::new(RwLock::new(make_device(name)))
}

#[allow(dead_code)]
fn no_op_emitter() -> EventEmitter {
    Arc::new(|_: &str, _: String| {})
}

#[tokio::test]
async fn full_handshake_between_client_and_server() {
    let (client_side, server_side) = tokio::io::duplex(4096);

    let peer_client = PeerId { id: "client".into(), device_id: None };
    let peer_server = PeerId { id: "server".into(), device_id: None };

    let state_server = Arc::new(RwLock::new(NetworkState::new()));
    let state_client = Arc::new(RwLock::new(NetworkState::new()));

    let server = RpcServerHandler::new(
        no_op_emitter(),
        shared_device("server-pc"),
        Arc::clone(&state_server) as Arc<dyn DeviceInfoStore>,
    );
    let client = RpcClientHandler::new(
        no_op_emitter(),
        shared_device("client-pc"),
        Arc::clone(&state_client) as Arc<dyn DeviceInfoStore>,
    );

    let (client_read, client_write) = tokio::io::split(client_side);
    let (server_read, server_write) = tokio::io::split(server_side);

    tokio::spawn(async move {
        let _ = server.handle(&peer_client, Box::new(server_write), Box::new(server_read)).await;
    });

    tokio::spawn(async move {
        let _ = client.handle(&peer_server, Box::new(client_write), Box::new(client_read)).await;
    });

    sleep(Duration::from_millis(50)).await;

    let server_state = state_server.read().await;
    let client_state = state_client.read().await;

    assert_eq!(
        server_state
            .get_device_info(&PeerId { id: "client".into(), device_id: None })
            .map(|d| d.name.as_str()),
        Some("client-pc"),
        "server did not receive DeviceInfo from client"
    );

    assert_eq!(
        client_state
            .get_device_info(&PeerId { id: "server".into(), device_id: None })
            .map(|d| d.name.as_str()),
        Some("server-pc"),
        "client did not receive DeviceInfo from server"
    );
}

#[tokio::test]
async fn server_fails_if_stream_closes_before_ping() {
    let (client_side, server_side) = tokio::io::duplex(4096);

    let state = Arc::new(RwLock::new(NetworkState::new()));
    let server = RpcServerHandler::new(
        no_op_emitter(),
        shared_device("server-pc"),
        Arc::clone(&state) as Arc<dyn DeviceInfoStore>,
    );

    let (server_read, server_write) = tokio::io::split(server_side);
    let server_task = tokio::spawn(async move {
        server.handle(&make_peer("client"), Box::new(server_write), Box::new(server_read)).await
    });

    drop(client_side);

    let result = server_task.await.unwrap();
    assert!(result.is_err(), "server should fail with closed stream");
    assert!(state.read().await.get_device_info(&make_peer("client")).is_none());
}

#[tokio::test]
async fn client_fails_if_stream_closes_before_pong() {
    let (client_side, server_side) = tokio::io::duplex(4096);

    let state = Arc::new(RwLock::new(NetworkState::new()));
    let client = RpcClientHandler::new(
        no_op_emitter(),
        shared_device("client-pc"),
        Arc::clone(&state) as Arc<dyn DeviceInfoStore>,
    );

    let (client_read, client_write) = tokio::io::split(client_side);
    let client_task = tokio::spawn(async move {
        client.handle(&make_peer("server"), Box::new(client_write), Box::new(client_read)).await
    });

    drop(server_side);

    let result = client_task.await.unwrap();
    assert!(result.is_err(), "client should fail with closed stream");
    assert!(state.read().await.get_device_info(&make_peer("server")).is_none());
}

#[tokio::test]
async fn neither_side_stores_device_info_if_connection_drops_before_exchange() {
    let (client_side, server_side) = tokio::io::duplex(4096);

    let state_server = Arc::new(RwLock::new(NetworkState::new()));
    let state_client = Arc::new(RwLock::new(NetworkState::new()));

    let server = RpcServerHandler::new(
        no_op_emitter(),
        shared_device("server-pc"),
        Arc::clone(&state_server) as Arc<dyn DeviceInfoStore>,
    );
    let client = RpcClientHandler::new(
        no_op_emitter(),
        shared_device("client-pc"),
        Arc::clone(&state_client) as Arc<dyn DeviceInfoStore>,
    );

    let (client_read, client_write) = tokio::io::split(client_side);
    let (server_read, server_write) = tokio::io::split(server_side);

    let server_task = tokio::spawn(async move {
        server.handle(&make_peer("client"), Box::new(server_write), Box::new(server_read)).await
    });

    let client_task = tokio::spawn(async move {
        client.handle(&make_peer("server"), Box::new(client_write), Box::new(client_read)).await
    });

    // Aborta os dois tasks antes de completar o exchange
    server_task.abort();
    client_task.abort();

    sleep(Duration::from_millis(20)).await;

    assert!(state_server.read().await.get_device_info(&make_peer("client")).is_none());
    assert!(state_client.read().await.get_device_info(&make_peer("server")).is_none());
}
