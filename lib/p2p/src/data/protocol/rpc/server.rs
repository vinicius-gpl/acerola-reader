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

pub struct RpcServerHandler {
    emit: EventEmitter,
    /// Compartilhado com `AcerolaP2p` (ver `set_local_device_name`) — lido de novo a cada
    /// handshake, não capturado por valor na construção, pra um apelido renomeado em runtime
    /// já valer na próxima sessão sem precisar reiniciar o node.
    local_info: Arc<RwLock<DeviceInfo>>,
    state: Arc<dyn DeviceInfoStore>,
}

#[async_trait]
impl ProtocolHandler for RpcServerHandler {
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

impl RpcServerHandler {
    pub fn new(
        emit: EventEmitter, local_info: Arc<RwLock<DeviceInfo>>, state: Arc<dyn DeviceInfoStore>,
    ) -> Self {
        Self { emit, local_info, state }
    }

    async fn perform_handshake(
        &self, peer: &PeerId, send: &mut Writer, recv: &mut Recv,
    ) -> Result<(), ConnectionError> {
        match read_byte(recv).await {
            Ok(PING) => {
                tracing::debug!(layer = "rpc_server", peer = %peer.id, "ping received");
                (self.emit)("rpc:ping_received", peer.id.clone());

                write_byte(send, PONG).await?;
                (self.emit)("rpc:pong_sent", peer.id.clone());
                Ok(())
            },
            Ok(_) => Err(ConnectionError::StreamFailed("unexpected byte before handshake".into())),
            Err(err) => Err(ConnectionError::from(err)),
        }
    }

    async fn exchange_device_info(
        &self, peer: &PeerId, send: &mut Writer, recv: &mut Recv,
    ) -> Result<(), ConnectionError> {
        let device_info = read_device_info(recv).await?;
        let local_info = self.local_info.read().await.clone();
        write_device_info(send, &local_info).await?;

        (self.emit)("rpc:device_info_exchanged", peer.id.clone());
        self.state.store_device_info(peer.clone(), device_info).await;

        Ok(())
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

    use super::{read_byte, write_byte, write_device_info, PING, PONG, *};
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
    async fn server_responds_pong_on_receiving_ping() {
        let (client_side, server_side) = tokio::io::duplex(4096);
        let (emit, events) = capture_emitter();
        let server = RpcServerHandler::new(emit, shared_device_info("server"), make_state());

        let peer = make_peer("peer-1");
        let peer_clone = peer.clone();
        tokio::spawn(async move {
            let (read, write) = tokio::io::split(server_side);
            let _ = server.handle(&peer_clone, Box::new(write), Box::new(read)).await;
        });

        let (read, write) = tokio::io::split(client_side);
        let mut recv = FramedRead::new(
            Box::new(read) as Box<dyn tokio::io::AsyncRead + Send + Unpin>,
            LengthDelimitedCodec::new(),
        );
        let mut send = FramedWrite::new(
            Box::new(write) as Box<dyn tokio::io::AsyncWrite + Send + Unpin>,
            LengthDelimitedCodec::new(),
        );

        write_byte(&mut send, PING).await.unwrap();
        let byte = read_byte(&mut recv).await.unwrap();
        assert_eq!(byte, PONG);

        write_device_info(&mut send, &make_device_info("client")).await.unwrap();
        sleep(Duration::from_millis(20)).await;

        assert!(events.lock().unwrap().iter().any(|event| event == "rpc:ping_received"));
        assert!(events.lock().unwrap().iter().any(|event| event == "rpc:pong_sent"));
    }

    #[tokio::test]
    async fn server_stores_client_device_info() {
        let (client_side, server_side) = tokio::io::duplex(4096);
        let (emit, _) = capture_emitter();
        let store = Arc::new(TestDeviceInfoStore::default());
        let server = RpcServerHandler::new(emit, shared_device_info("server"), store.clone());

        let peer = make_peer("peer-1");
        let peer_clone = peer.clone();
        tokio::spawn(async move {
            let (read, write) = tokio::io::split(server_side);
            let _ = server.handle(&peer_clone, Box::new(write), Box::new(read)).await;
        });

        let (read, write) = tokio::io::split(client_side);
        let mut recv = FramedRead::new(
            Box::new(read) as Box<dyn tokio::io::AsyncRead + Send + Unpin>,
            LengthDelimitedCodec::new(),
        );
        let mut send = FramedWrite::new(
            Box::new(write) as Box<dyn tokio::io::AsyncWrite + Send + Unpin>,
            LengthDelimitedCodec::new(),
        );

        write_byte(&mut send, PING).await.unwrap();
        read_byte(&mut recv).await.unwrap();
        write_device_info(&mut send, &make_device_info("client-pc")).await.unwrap();

        sleep(Duration::from_millis(30)).await;

        let stored = store.devices.lock().unwrap().get(&peer).map(|info| info.name.clone());
        assert_eq!(stored, Some("client-pc".to_string()));
    }

    /// Renomear o dispositivo local (`AcerolaP2p::set_local_device_name`, que escreve no mesmo
    /// `Arc<RwLock<DeviceInfo>>` passado aqui) tem que valer no próximo handshake mesmo sem
    /// reconstruir o handler — é essa propriedade que permite o apelido aplicar na hora, sem
    /// reiniciar o app.
    #[tokio::test]
    async fn exchange_uses_current_local_info_after_runtime_rename() {
        let (client_side, server_side) = tokio::io::duplex(4096);
        let (emit, _) = capture_emitter();
        let local_info = shared_device_info("server-old-name");
        let server = RpcServerHandler::new(emit, Arc::clone(&local_info), make_state());

        local_info.write().await.name = "server-new-name".to_string();

        let peer = make_peer("peer-1");
        tokio::spawn(async move {
            let (read, write) = tokio::io::split(server_side);
            let _ = server.handle(&peer, Box::new(write), Box::new(read)).await;
        });

        let (read, write) = tokio::io::split(client_side);
        let mut recv = FramedRead::new(
            Box::new(read) as Box<dyn tokio::io::AsyncRead + Send + Unpin>,
            LengthDelimitedCodec::new(),
        );
        let mut send = FramedWrite::new(
            Box::new(write) as Box<dyn tokio::io::AsyncWrite + Send + Unpin>,
            LengthDelimitedCodec::new(),
        );

        write_byte(&mut send, PING).await.unwrap();
        read_byte(&mut recv).await.unwrap();
        write_device_info(&mut send, &make_device_info("client")).await.unwrap();

        let received = read_device_info(&mut recv).await.unwrap();
        assert_eq!(received.name, "server-new-name");
    }

    #[tokio::test]
    async fn server_terminates_if_ping_not_received() {
        let (client_side, server_side) = tokio::io::duplex(4096);
        let (emit, _) = capture_emitter();
        let server = RpcServerHandler::new(emit, shared_device_info("server"), make_state());

        let peer = make_peer("peer-1");
        let server_task = tokio::spawn(async move {
            let (read, write) = tokio::io::split(server_side);
            server.handle(&peer, Box::new(write), Box::new(read)).await
        });

        drop(client_side);
        sleep(Duration::from_millis(20)).await;
        assert!(server_task.is_finished());
    }

    /// Regressão: o handshake é uma troca pontual, não uma sessão que precisa ficar viva.
    /// A liveness da conexão física já é responsabilidade do keepalive nativo do QUIC/iroh, não
    /// deste RPC — então `handle()` deve retornar assim que a troca de device info terminar, sem
    /// esperar por nenhum PING/GOODBYE adicional.
    #[tokio::test]
    async fn handle_returns_ok_immediately_after_device_info_exchange() {
        let (client_side, server_side) = tokio::io::duplex(4096);
        let (emit, _) = capture_emitter();
        let server = RpcServerHandler::new(emit, shared_device_info("server"), make_state());

        let peer = make_peer("peer-1");
        let server_task = tokio::spawn(async move {
            let (read, write) = tokio::io::split(server_side);
            server.handle(&peer, Box::new(write), Box::new(read)).await
        });

        let (read, write) = tokio::io::split(client_side);
        let mut recv = FramedRead::new(
            Box::new(read) as Box<dyn tokio::io::AsyncRead + Send + Unpin>,
            LengthDelimitedCodec::new(),
        );
        let mut send = FramedWrite::new(
            Box::new(write) as Box<dyn tokio::io::AsyncWrite + Send + Unpin>,
            LengthDelimitedCodec::new(),
        );

        write_byte(&mut send, PING).await.unwrap();
        read_byte(&mut recv).await.unwrap();
        write_device_info(&mut send, &make_device_info("client")).await.unwrap();

        let result = tokio::time::timeout(Duration::from_millis(200), server_task).await;
        assert!(result.is_ok(), "handle() should return promptly, not hang waiting for GOODBYE");
        assert!(result.unwrap().unwrap().is_ok());
    }
}
