use std::sync::Arc;

use async_trait::async_trait;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::RwLock,
};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use super::{read_byte, read_device_info, write_byte, write_device_info, Recv, Writer, PING, PONG};
use crate::{
    data::{
        identity::device_info::DeviceInfo,
        protocol::{DeviceInfoStore, EventEmitter, ProtocolHandler},
    },
    infra::{error::ConnectionError, peer::PeerId},
};

pub struct RpcClientHandler {
    emit: EventEmitter,
    /// Compartilhado com `AcerolaP2p` (ver `set_local_device_name`) — mesma razão do campo
    /// homônimo em `RpcServerHandler`.
    local_info: Arc<RwLock<DeviceInfo>>,
    state: Arc<dyn DeviceInfoStore>,
}

#[async_trait]
impl ProtocolHandler for RpcClientHandler {
    async fn handle(
        &self, peer: &PeerId, send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), ConnectionError> {
        let mut framed_recv = FramedRead::new(recv, LengthDelimitedCodec::new());
        let mut framed_send = FramedWrite::new(send, LengthDelimitedCodec::new());

        self.perform_handshake(peer, &mut framed_send, &mut framed_recv).await?;
        self.exchange_device_info(peer, &mut framed_send, &mut framed_recv).await?;

        Ok(())
    }
}

impl RpcClientHandler {
    pub fn new(
        emit: EventEmitter, local_info: Arc<RwLock<DeviceInfo>>, state: Arc<dyn DeviceInfoStore>,
    ) -> Self {
        Self { emit, local_info, state }
    }

    async fn perform_handshake(
        &self, peer: &PeerId, send: &mut Writer, recv: &mut Recv,
    ) -> Result<(), ConnectionError> {
        write_byte(send, PING).await?;
        (self.emit)("rpc:ping_sent", peer.id.clone());

        match read_byte(recv).await {
            Ok(PONG) => {
                tracing::debug!(layer = "rpc_client", peer = %peer.id, "pong received");
                (self.emit)("rpc:pong_received", peer.id.clone());
                Ok(())
            },
            Ok(_) => Err(ConnectionError::StreamFailed("unexpected response to ping".into())),
            Err(err) => Err(ConnectionError::from(err)),
        }
    }

    async fn exchange_device_info(
        &self, peer: &PeerId, send: &mut Writer, recv: &mut Recv,
    ) -> Result<(), ConnectionError> {
        let local_info = self.local_info.read().await.clone();
        write_device_info(send, &local_info).await?;
        (self.emit)("rpc:device_info_sent", peer.id.clone());

        match read_device_info(recv).await {
            Ok(device_info) => {
                tracing::debug!(layer = "rpc_client", peer = %peer.id, "device info received");

                (self.emit)("rpc:device_info_received", peer.id.clone());
                self.state.store_device_info(peer.clone(), device_info).await;

                Ok(())
            },
            Err(err) => Err(ConnectionError::from(err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use tokio::time::{sleep, Duration};
    use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

    use super::{
        super::{read_byte, read_device_info, write_byte, write_device_info, PING, PONG},
        *,
    };
    use crate::{data::protocol::EventEmitter, infra::peer::PeerId};

    #[derive(Default)]
    struct TestDeviceInfoStore {
        devices: Arc<Mutex<HashMap<PeerId, DeviceInfo>>>,
    }

    #[async_trait]
    impl DeviceInfoStore for TestDeviceInfoStore {
        async fn store_device_info(&self, peer: PeerId, info: DeviceInfo) {
            self.devices.lock().unwrap().insert(peer, info);
        }
    }

    fn make_peer(id: &str) -> PeerId {
        PeerId { id: id.to_string(), device_id: None }
    }

    fn make_device_info(name: &str) -> DeviceInfo {
        DeviceInfo { name: name.to_string(), os: "linux".to_string(), version: "0.0.1".to_string() }
    }

    fn shared_device_info(name: &str) -> Arc<RwLock<DeviceInfo>> {
        Arc::new(RwLock::new(make_device_info(name)))
    }

    fn make_state() -> Arc<dyn DeviceInfoStore> {
        Arc::new(TestDeviceInfoStore::default())
    }

    fn capture_emitter() -> (EventEmitter, Arc<Mutex<Vec<String>>>) {
        let events = Arc::new(Mutex::new(Vec::new()));
        let clone = Arc::clone(&events);

        let emit: EventEmitter = Arc::new(move |event: &str, _: String| {
            clone.lock().unwrap().push(event.to_string());
        });

        (emit, events)
    }

    #[tokio::test]
    async fn client_sends_ping_first() {
        let (client_side, server_side) = tokio::io::duplex(4096);
        let (emit, events) = capture_emitter();
        let client = RpcClientHandler::new(emit, shared_device_info("client"), make_state());

        let peer = make_peer("peer-1");
        let peer_clone = peer.clone();
        tokio::spawn(async move {
            let (read, write) = tokio::io::split(client_side);
            let _ = client.handle(&peer_clone, Box::new(write), Box::new(read)).await;
        });

        let (read, write) = tokio::io::split(server_side);
        let mut recv = FramedRead::new(
            Box::new(read) as Box<dyn tokio::io::AsyncRead + Send + Unpin>,
            LengthDelimitedCodec::new(),
        );
        let mut send = FramedWrite::new(
            Box::new(write) as Box<dyn tokio::io::AsyncWrite + Send + Unpin>,
            LengthDelimitedCodec::new(),
        );

        let byte = read_byte(&mut recv).await.unwrap();
        assert_eq!(byte, PING);

        write_byte(&mut send, PONG).await.unwrap();
        write_device_info(&mut send, &make_device_info("server")).await.unwrap();
        read_device_info(&mut recv).await.unwrap();

        sleep(Duration::from_millis(20)).await;
        assert!(events.lock().unwrap().iter().any(|event| event == "rpc:ping_sent"));
        assert!(events.lock().unwrap().iter().any(|event| event == "rpc:pong_received"));
    }

    #[tokio::test]
    async fn client_terminates_if_wrong_byte_received_after_ping() {
        let (client_side, server_side) = tokio::io::duplex(4096);
        let (emit, _) = capture_emitter();
        let client = RpcClientHandler::new(emit, shared_device_info("client"), make_state());

        let peer = make_peer("peer-1");
        let client_task = tokio::spawn(async move {
            let (read, write) = tokio::io::split(client_side);
            client.handle(&peer, Box::new(write), Box::new(read)).await
        });

        let (read, write) = tokio::io::split(server_side);
        let mut recv = FramedRead::new(
            Box::new(read) as Box<dyn tokio::io::AsyncRead + Send + Unpin>,
            LengthDelimitedCodec::new(),
        );
        let mut send = FramedWrite::new(
            Box::new(write) as Box<dyn tokio::io::AsyncWrite + Send + Unpin>,
            LengthDelimitedCodec::new(),
        );

        read_byte(&mut recv).await.unwrap();
        write_byte(&mut send, 0xFF).await.unwrap();

        sleep(Duration::from_millis(20)).await;
        assert!(client_task.is_finished());
        assert!(client_task.await.unwrap().is_err());
    }

    #[tokio::test]
    async fn client_stores_device_info_from_server() {
        let (client_side, server_side) = tokio::io::duplex(4096);
        let (emit, _) = capture_emitter();
        let store = Arc::new(TestDeviceInfoStore::default());
        let client = RpcClientHandler::new(emit, shared_device_info("client"), store.clone());

        let peer = make_peer("peer-1");
        let peer_clone = peer.clone();
        tokio::spawn(async move {
            let (read, write) = tokio::io::split(client_side);
            let _ = client.handle(&peer_clone, Box::new(write), Box::new(read)).await;
        });

        let (read, write) = tokio::io::split(server_side);
        let mut recv = FramedRead::new(
            Box::new(read) as Box<dyn tokio::io::AsyncRead + Send + Unpin>,
            LengthDelimitedCodec::new(),
        );
        let mut send = FramedWrite::new(
            Box::new(write) as Box<dyn tokio::io::AsyncWrite + Send + Unpin>,
            LengthDelimitedCodec::new(),
        );

        read_byte(&mut recv).await.unwrap();
        write_byte(&mut send, PONG).await.unwrap();
        write_device_info(&mut send, &make_device_info("server-pc")).await.unwrap();
        read_device_info(&mut recv).await.unwrap();

        sleep(Duration::from_millis(30)).await;

        let stored = store.devices.lock().unwrap().get(&peer).map(|info| info.name.clone());
        assert_eq!(stored, Some("server-pc".to_string()));
    }

    /// Regressão: o handshake é uma troca pontual, não uma sessão que precisa ficar viva.
    /// A liveness da conexão física já é responsabilidade do keepalive nativo do QUIC/iroh, não
    /// deste RPC — então `handle()` deve retornar assim que a troca de device info terminar, sem
    /// esperar por nenhum PING/GOODBYE adicional.
    #[tokio::test]
    async fn handle_returns_ok_immediately_after_device_info_exchange() {
        let (client_side, server_side) = tokio::io::duplex(4096);
        let (emit, _) = capture_emitter();
        let client = RpcClientHandler::new(emit, shared_device_info("client"), make_state());

        let peer = make_peer("peer-1");
        let client_task = tokio::spawn(async move {
            let (read, write) = tokio::io::split(client_side);
            client.handle(&peer, Box::new(write), Box::new(read)).await
        });

        let (read, write) = tokio::io::split(server_side);
        let mut recv = FramedRead::new(
            Box::new(read) as Box<dyn tokio::io::AsyncRead + Send + Unpin>,
            LengthDelimitedCodec::new(),
        );
        let mut send = FramedWrite::new(
            Box::new(write) as Box<dyn tokio::io::AsyncWrite + Send + Unpin>,
            LengthDelimitedCodec::new(),
        );

        read_byte(&mut recv).await.unwrap();
        write_byte(&mut send, PONG).await.unwrap();
        write_device_info(&mut send, &make_device_info("server")).await.unwrap();
        read_device_info(&mut recv).await.unwrap();

        let result = tokio::time::timeout(Duration::from_millis(200), client_task).await;
        assert!(result.is_ok(), "handle() should return promptly, not hang waiting for GOODBYE");
        assert!(result.unwrap().unwrap().is_ok());
    }
}
