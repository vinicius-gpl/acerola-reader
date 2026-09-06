//! Stub de Transporte implementado para mockar conexões e isolar o unit-test do NetworkManager.
//!
//! Provê o `MockTransport` e a sua respectiva manivela (`MockTransportHandle`), que
//! permite injetar conexões programaticamente (como pipes duplexados na memória)
//! simulando solicitações reais que viriam da rede exterior.
//!
//! O `open_bi` também é suportado via `MockTransportHandle::expect_open`, permitindo
//! pré-registrar streams que serão devolvidas ao chamador quando o transporte discar.

use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use tokio::{
    io::{AsyncRead, AsyncWrite, DuplexStream},
    sync::{mpsc, Mutex},
};

use crate::{
    core::transport::{IncomingConnection, P2pTransport},
    infra::{
        error::ConnectionError,
        peer::{PeerAddr, PeerId},
    },
};

/// Assinatura interna que empacota as propriedades forjadas de uma nova "conexão P2P".
#[rustfmt::skip]
type InjectedConnection = (Vec<u8>, PeerId, Box<dyn AsyncWrite + Send + Unpin>, Box<dyn AsyncRead + Send + Unpin>);

/// Assinatura interna de uma resposta pré-registrada para `open_bi`.
type OutboundConnection = (Box<dyn AsyncWrite + Send + Unpin>, Box<dyn AsyncRead + Send + Unpin>);

/// Conexão simulada representando um par de streams já atreladas a um nó fictício.
struct MockIncoming {
    alpn: Vec<u8>,
    peer: PeerId,
    addr: PeerAddr,
    send: Box<dyn AsyncWrite + Send + Unpin>,
    recv: Box<dyn AsyncRead + Send + Unpin>,
}

#[async_trait]
impl IncomingConnection for MockIncoming {
    fn alpn(&self) -> &[u8] {
        &self.alpn
    }

    fn peer(&self) -> &PeerId {
        &self.peer
    }

    fn addr(&self) -> &PeerAddr {
        &self.addr
    }

    async fn accept_bi(
        self: Box<Self>,
    ) -> Result<
        (Box<dyn AsyncWrite + Send + Unpin>, Box<dyn AsyncRead + Send + Unpin>),
        ConnectionError,
    > {
        Ok((self.send, self.recv))
    }
}

use std::{collections::HashMap, sync::Arc};

/// Implementador falso (Dummy) da trait `P2pTransport`.
/// O NetworkManager irá ficar suspenso esperando conexões que o handle submete pelo buffer.
pub struct MockTransport {
    inbound_rx: Mutex<mpsc::UnboundedReceiver<InjectedConnection>>,
    outbound_rx: Mutex<mpsc::UnboundedReceiver<OutboundConnection>>,
    latencies: Arc<Mutex<HashMap<PeerId, std::time::Duration>>>,
    /// Compartilhado com `MockTransportHandle::was_shutdown_called` — prova que
    /// `NetworkManager::run()` chama `transport.shutdown()` de verdade no `NetworkCommand::Shutdown`,
    /// em vez de só sair do loop sem desligar o transporte por baixo (era exatamente esse o bug:
    /// o `Endpoint` real do iroh nunca fechava, ver `IrohTransport::shutdown`).
    shutdown_called: Arc<AtomicBool>,
}

/// A manivela para disparar streams pra dentro do ambiente mockado.
pub struct MockTransportHandle {
    inbound_tx: mpsc::UnboundedSender<InjectedConnection>,
    #[allow(dead_code)]
    outbound_tx: mpsc::UnboundedSender<OutboundConnection>,
    latencies: Arc<Mutex<HashMap<PeerId, std::time::Duration>>>,
    shutdown_called: Arc<AtomicBool>,
}

impl MockTransportHandle {
    /// Simula um peer remoto iniciando uma conexão de entrada.
    pub fn inject(&self, alpn: &[u8], peer: PeerId, client: DuplexStream, server: DuplexStream) {
        let _ = self.inbound_tx.send((alpn.to_vec(), peer, Box::new(server), Box::new(client)));
    }

    /// Pré-registra streams que serão devolvidas na próxima chamada a `open_bi`.
    #[allow(dead_code)]
    pub fn expect_open(&self, client: DuplexStream, server: DuplexStream) {
        let _ = self.outbound_tx.send((Box::new(client), Box::new(server)));
    }

    /// Configura a latência mockada para um peer.
    #[allow(dead_code)]
    pub async fn set_latency(&self, peer: PeerId, latency: std::time::Duration) {
        self.latencies.lock().await.insert(peer, latency);
    }

    /// `true` se `MockTransport::shutdown()` já foi chamado ao menos uma vez.
    #[allow(dead_code)]
    pub fn was_shutdown_called(&self) -> bool {
        self.shutdown_called.load(Ordering::SeqCst)
    }
}

/// Construtor emparelhado que devolve tanto o Transporte Fictício (para injetar no Builder)
/// quanto o Handle (para ser usado pelo desenvolvedor nos scripts de teste).
pub fn mock_transport() -> (MockTransport, MockTransportHandle) {
    let (inbound_tx, inbound_rx) = mpsc::unbounded_channel();
    let (outbound_tx, outbound_rx) = mpsc::unbounded_channel();
    let latencies = Arc::new(Mutex::new(HashMap::new()));
    let shutdown_called = Arc::new(AtomicBool::new(false));
    (
        MockTransport {
            inbound_rx: Mutex::new(inbound_rx),
            outbound_rx: Mutex::new(outbound_rx),
            latencies: Arc::clone(&latencies),
            shutdown_called: Arc::clone(&shutdown_called),
        },
        MockTransportHandle { inbound_tx, outbound_tx, latencies, shutdown_called },
    )
}

#[async_trait]
impl P2pTransport for MockTransport {
    fn local_id(&self) -> PeerId {
        PeerId { id: "mock-peer".to_string(), device_id: None }
    }

    fn local_addr(&self) -> Result<PeerAddr, ConnectionError> {
        Ok(PeerAddr { id: self.local_id(), addrs: vec![] })
    }

    async fn accept(&self) -> Result<Box<dyn IncomingConnection>, ConnectionError> {
        let (alpn, peer, send, recv) =
            self.inbound_rx.lock().await.recv().await.ok_or(ConnectionError::Shutdown)?;

        let addr = PeerAddr { id: peer.clone(), addrs: vec![] };

        Ok(Box::new(MockIncoming { alpn, peer, addr, send, recv }))
    }

    #[rustfmt::skip]
    async fn open_bi(
        &self, _alpn: &[u8], _peer: &PeerAddr,
    ) -> Result<
        (Box<dyn AsyncWrite + Send + Unpin>, Box<dyn AsyncRead + Send + Unpin>),
        ConnectionError,
    > {
        self.outbound_rx.lock().await.recv().await.ok_or(ConnectionError::Shutdown)
    }

    async fn latency(&self, peer: &PeerId) -> Option<std::time::Duration> {
        self.latencies.lock().await.get(peer).cloned()
    }

    async fn shutdown(&self) -> Result<(), ConnectionError> {
        self.shutdown_called.store(true, Ordering::SeqCst);
        Ok(())
    }
}
