use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use iroh::{
    endpoint::{Connection, IncomingAddr},
    Endpoint, EndpointAddr, EndpointId,
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::{mpsc, Mutex, RwLock},
};

use super::{
    blobs_bridge::BlobsIntegration,
    connection::{ConnectionReader, ConnectionWriter, IrohIncoming},
};
use crate::{
    core::{
        blobs::P2pBlobStore,
        transport::{IncomingConnection, P2pTransport},
    },
    infra::{
        error::ConnectionError,
        peer::{PeerAddr, PeerId},
    },
};

/// Chave de cache: uma conexão física é específica de um par (peer, ALPN), já que o ALPN é
/// negociado uma única vez no handshake TLS/QUIC — dois protocolos distintos para o mesmo peer
/// sempre habitam conexões QUIC diferentes, mesmo que simultâneas.
type ConnectionKey = (PeerId, Vec<u8>);

/// Interface concreta que gerencia o Endpoint UDP local e a configuração de chaves usando a suite Iroh.
///
/// `connections` é um pool de conexões físicas vivas, reaproveitadas entre chamadas — não um
/// registro passivo. QUIC (e o iroh) é feito para isso: o handshake (que envolve NAT-traversal e
/// pode negociar multipath) é a parte cara; abrir/fechar streams em cima de uma conexão já
/// estabelecida é essencialmente grátis. Por isso guardamos aqui uma referência *forte* de
/// propósito — ela é o que mantém a conexão viva entre uma sessão de protocolo e a próxima, para
/// `open_bi` reaproveitar em vez de discar do zero a cada chamada.
///
/// Como QUIC é full-duplex, uma entrada pode vir tanto de um `open_bi` (nós discamos) quanto de
/// um `accept` (o peer discou) — para efeito de reaproveitamento futuro tanto faz quem iniciou.
///
/// Entradas mortas (peer caiu, idle timeout do próprio iroh expirou) só são removidas
/// preguiçosamente, na próxima vez que `open_bi`/`latency` tocarem naquela chave. Isso é
/// aceitável: uma conexão já fechada não segura nenhum path de NAT-traversal nem handshake
/// ativo — o resíduo é só a entrada do HashMap em si, não o vazamento de recurso de rede que
/// existia antes (quando cada chamada deixava uma conexão física inteira presa para sempre).
pub struct IrohTransport {
    endpoint: Endpoint,
    /// `true` quando o modo de relay resolvido é `Disabled` (`RelayModeConfig::MdnsOnly`, ou o
    /// legado sem nenhuma URL) — restringe dial (`open_bi`) e accept (`drive_incoming_connections`)
    /// a endereços de rede local. Sem isso, um `EndpointAddr` cacheado de um pareamento anterior
    /// (de quando um relay ainda estava ativo) ou um mapeamento de NAT ainda vivo no roteador
    /// continuam permitindo conexão cross-internet mesmo com "só rede local" selecionado — o
    /// `RelayMode` do iroh por si só não fecha a porta UDP nem filtra endereços cacheados.
    restrict_to_lan: bool,
    connections: Arc<RwLock<HashMap<ConnectionKey, Connection>>>,
    /// Um mutex por `(peer, alpn)` que ainda está sendo discado, usado para serializar a
    /// sequência `check cache → dial → insert cache` em `open_bi`. Sem isso, duas chamadas
    /// concorrentes para a mesma chave podem ver o cache vazio simultaneamente (o `.await` de
    /// `endpoint.connect` não segura nenhum lock) e disparar duas conexões QUIC físicas
    /// distintas para o mesmo peer. Com o lock por chave, a segunda chamada aguarda a primeira
    /// terminar de discar e reaproveita o resultado dela via o double-check após adquirir o
    /// lock. Entradas não são removidas depois de usadas — mesmo racional de limpeza
    /// preguiçosa que `connections` já usa.
    dial_locks: Mutex<HashMap<ConnectionKey, Arc<tokio::sync::Mutex<()>>>>,
    /// Streams inbound já aceitas, entregues por `drive_incoming_connections`/`drive_incoming_streams`
    /// (tasks de fundo). `accept()` só consome este canal — ver o doc dessas funções para o motivo
    /// de existirem: uma conexão reaproveitada pode receber várias streams ao longo da vida dela,
    /// não só uma, então algo precisa continuar escutando cada conexão depois do primeiro stream.
    incoming_rx: Mutex<mpsc::UnboundedReceiver<Box<dyn IncomingConnection>>>,
    /// Integração opcional com o adapter de blobs — "null object" quando a feature
    /// `iroh-blobs-adapter` está desligada, ver `blobs_bridge.rs`.
    blobs: BlobsIntegration,
}

impl IrohTransport {
    pub(crate) fn new(endpoint: Endpoint, blobs: BlobsIntegration, restrict_to_lan: bool) -> Self {
        let connections: Arc<RwLock<HashMap<ConnectionKey, Connection>>> =
            Arc::new(RwLock::new(HashMap::new()));
        let (incoming_tx, incoming_rx) = mpsc::unbounded_channel();

        tokio::spawn(drive_incoming_connections(
            endpoint.clone(),
            Arc::clone(&connections),
            incoming_tx,
            blobs.clone(),
            restrict_to_lan,
        ));

        Self {
            endpoint,
            restrict_to_lan,
            connections,
            dial_locks: Mutex::new(HashMap::new()),
            incoming_rx: Mutex::new(incoming_rx),
            blobs,
        }
    }

    /// Disca (ou reaproveita) a conexão física para `key`, garantindo que no máximo um
    /// `endpoint.connect()` esteja em voo por vez para essa chave. Chamadas concorrentes para a
    /// mesma `key` esperam no mutex por-chave e, ao adquiri-lo, primeiro reconferem o cache —
    /// a chamada que discou primeiro já pode ter inserido a conexão enquanto esta esperava.
    async fn dial_or_reuse(
        &self, key: &ConnectionKey, addr: &EndpointAddr, alpn: &[u8], peer: &PeerId,
    ) -> Result<Connection, ConnectionError> {
        let key_lock = {
            let mut locks = self.dial_locks.lock().await;
            Arc::clone(
                locks.entry(key.clone()).or_insert_with(|| Arc::new(tokio::sync::Mutex::new(()))),
            )
        };

        let _dial_guard = key_lock.lock().await;

        if let Some(conn) = self.connections.read().await.get(key).cloned() {
            if conn.close_reason().is_none() {
                return Ok(conn);
            }
        }

        tracing::debug!(
            peer = %peer.id,
            layer = "iroh_transport",
            alpn = ?String::from_utf8_lossy(alpn),
            "initiating outbound connection"
        );

        let conn = self.endpoint.connect(addr.clone(), alpn).await?;
        self.connections.write().await.insert(key.clone(), conn.clone());
        Ok(conn)
    }

    /// Trata a conversão sintática das Strings em NodeIds estritos nativos do iroh.
    #[rustfmt::skip]
    fn peer_to_addr(&self, peer: &PeerId) -> Result<EndpointAddr, ConnectionError> {
        let id: EndpointId = peer.id.parse().map_err(|_| ConnectionError::PeerNotFound(PeerId { id: peer.id.clone(), device_id: None }))?;
        Ok(EndpointAddr::from(id))
    }

    pub async fn latency(&self, peer: &PeerId) -> Option<std::time::Duration> {
        P2pTransport::latency(self, peer).await
    }
}

#[async_trait]
impl P2pTransport for IrohTransport {
    fn local_id(&self) -> PeerId {
        to_peer_id(self.endpoint.id())
    }

    fn local_addr(&self) -> Result<PeerAddr, ConnectionError> {
        to_peer_addr(self.endpoint.id(), self.endpoint.addr())
    }

    async fn accept(&self) -> Result<Box<dyn IncomingConnection>, ConnectionError> {
        self.incoming_rx.lock().await.recv().await.ok_or(ConnectionError::Shutdown)
    }

    async fn open_bi(
        &self, alpn: &[u8], peer: &PeerAddr,
    ) -> Result<
        (Box<dyn AsyncWrite + Send + Unpin>, Box<dyn AsyncRead + Send + Unpin>),
        ConnectionError,
    > {
        let addr = if peer.addrs.is_empty() {
            self.peer_to_addr(&peer.id)?
        } else {
            serde_json::from_slice(&peer.addrs)
                .map_err(|_| ConnectionError::PeerNotFound(peer.id.clone()))?
        };

        // Sem relay algum configurado, um `EndpointAddr` cacheado de ANTES do usuário desligar o
        // relay pode carregar um IP público (ou hint de relay) que ainda resolve — descartar
        // essas pistas aqui garante que "só rede local" também vale pro lado que disca, não só
        // pro que aceita.
        let addr = if self.restrict_to_lan { restrict_to_lan_addr(addr) } else { addr };

        let key = (peer.id.clone(), alpn.to_vec());

        // Até duas tentativas: a primeira reaproveita uma conexão viva já cacheada para este
        // peer+ALPN (discada por uma chamada anterior, ou aceita via `accept` — QUIC é
        // full-duplex, então tanto faz quem iniciou). Se essa conexão já estiver morta sem que
        // `close_reason()` ainda reflita isso (ex: peer caiu sem handshake de fechamento), a
        // segunda tentativa descarta a entrada e disca do zero.
        let mut retried = false;
        loop {
            let cached = self.connections.read().await.get(&key).cloned();

            let conn = match cached {
                Some(conn) if conn.close_reason().is_none() => conn,
                _ => self.dial_or_reuse(&key, &addr, alpn, &peer.id).await?,
            };

            match conn.open_bi().await {
                Ok((send, recv)) => {
                    tracing::trace!(
                        peer = %peer.id,
                        layer = "iroh_transport",
                        "outbound bi-stream opened"
                    );

                    let shared_conn = Arc::new(conn);

                    return Ok((
                        Box::new(ConnectionWriter::new(send, Arc::clone(&shared_conn))),
                        Box::new(ConnectionReader::new(recv, shared_conn)),
                    ));
                },
                Err(_) if !retried => {
                    tracing::debug!(
                        peer = %peer.id,
                        layer = "iroh_transport",
                        "cached connection is dead, discarding and dialing fresh"
                    );
                    self.connections.write().await.remove(&key);
                    retried = true;
                },
                Err(err) => return Err(err.into()),
            }
        }
    }

    async fn latency(&self, peer: &PeerId) -> Option<std::time::Duration> {
        // Um peer pode ter mais de uma conexão física cacheada (uma por ALPN em uso). Descarta
        // no caminho quaisquer entradas já mortas (peer caiu, idle timeout expirou) e usa a
        // primeira ainda viva encontrada.
        let mut guard = self.connections.write().await;

        let dead_keys: Vec<ConnectionKey> = guard
            .iter()
            .filter(|(key, conn)| key.0 == *peer && conn.close_reason().is_some())
            .map(|(key, _)| key.clone())
            .collect();

        for key in &dead_keys {
            guard.remove(key);
        }

        let conn = guard.iter().find(|(key, _)| key.0 == *peer).map(|(_, conn)| conn.clone())?;

        let path_list = conn.paths();

        // Prefere o caminho selecionado; aceita qualquer outro como fallback.
        path_list
            .iter()
            .filter(|path| path.is_selected())
            .chain(path_list.iter())
            .map(|path| path.rtt())
            .next()
    }

    async fn blobs(&self) -> Option<Arc<dyn P2pBlobStore>> {
        self.blobs.as_capability()
    }

    async fn is_connected(&self, peer: &PeerId) -> bool {
        self.connections
            .read()
            .await
            .iter()
            .any(|(key, conn)| key.0 == *peer && conn.close_reason().is_none())
    }

    /// Delega para `Endpoint::network_change()` — ver a doc em `P2pTransport::network_change`
    /// para o porquê disso ser necessário no Android.
    async fn network_change(&self) {
        self.endpoint.network_change().await;
    }

    async fn invalidate(&self, peer: &PeerId, alpn: &[u8]) {
        let key = (peer.clone(), alpn.to_vec());
        self.connections.write().await.remove(&key);
    }

    /// Executa o teardown forçado do componente iroh.
    ///
    /// Warn: O endpoint é compartilhado em formato Arc no backend do crate `iroh`.
    /// Desligar essa faceta pode necessitar dropar todos os componentes de leitura remanescentes.
    async fn shutdown(&self) -> Result<(), ConnectionError> {
        self.endpoint.close().await;
        Ok(())
    }
}

/// Converte um ID nativo do Iroh para o PeerId da nossa abstração.
fn to_peer_id(node_id: EndpointId) -> PeerId {
    PeerId::from_public_key(node_id.to_string(), node_id.as_bytes())
}

/// Converte um par ID+Endereço do Iroh para o PeerAddr da nossa abstração.
fn to_peer_addr(node_id: EndpointId, addr: EndpointAddr) -> Result<PeerAddr, ConnectionError> {
    let addrs = serde_json::to_vec(&addr)
        .map_err(|error| ConnectionError::StreamFailed(error.to_string()))?;
    Ok(PeerAddr { id: to_peer_id(node_id), addrs })
}

/// Task de fundo (uma por `IrohTransport`): aceita conexões físicas novas do endpoint e, para
/// cada uma, dispara um loop próprio (`drive_incoming_streams`) que continua aceitando streams
/// naquela conexão enquanto ela estiver viva.
///
/// Isso existe porque uma conexão pode ser reaproveitada (ver o cache em `open_bi`/`accept`) e
/// receber várias streams ao longo da vida dela — se só reagíssemos ao primeiro `accept_bi`,
/// como antes, o `NetworkManager` nunca ficaria sabendo de uma segunda stream aberta pelo peer
/// numa conexão já estabelecida.
async fn drive_incoming_connections(
    endpoint: Endpoint, connections: Arc<RwLock<HashMap<ConnectionKey, Connection>>>,
    incoming_tx: mpsc::UnboundedSender<Box<dyn IncomingConnection>>, blobs: BlobsIntegration,
    restrict_to_lan: bool,
) {
    loop {
        let Some(incoming) = endpoint.accept().await else { break };
        let incoming_addr = incoming.remote_addr();

        tracing::trace!(layer = "iroh_transport", "incoming connection request received");

        let conn = match incoming.await {
            Ok(conn) => conn,
            Err(err) => {
                tracing::debug!(layer = "iroh_transport", error = ?err, "inbound handshake failed");
                continue;
            },
        };

        let remote_id = conn.remote_id();
        let alpn = conn.alpn().to_vec();

        // Sem relay algum configurado, a porta UDP local segue aberta pra qualquer pacote que
        // chegue — um endereço direto cacheado de ANTES do relay ser desligado, ou um
        // mapeamento de NAT ainda vivo no roteador, deixariam essa conexão passar mesmo com "só
        // rede local" selecionado. `RelayMode::Disabled` por si só não filtra isso.
        if restrict_to_lan && !is_private_incoming(&incoming_addr) {
            tracing::warn!(
                peer = %remote_id,
                layer = "iroh_transport",
                incoming_addr = ?incoming_addr,
                "rejecting inbound connection from non-LAN address while relay is disabled"
            );
            conn.close(iroh::endpoint::VarInt::from_u32(0), b"relay disabled: only LAN peers allowed");
            continue;
        }

        // O ALPN de blobs não passa pelo dispatch genérico por ALPN registrado no
        // `NetworkManager` — o `BlobsProtocol` real do iroh-blobs precisa da `Connection`
        // inteira (não de um stream já isolado via `accept_bi`), então é atendido aqui.
        if blobs.try_accept(&alpn, &conn).await {
            continue;
        }

        let endpoint_addr = resolve_endpoint_addr(remote_id, &incoming_addr);

        let peer = to_peer_id(remote_id);
        let addr = match to_peer_addr(remote_id, endpoint_addr) {
            Ok(addr) => addr,
            Err(err) => {
                tracing::debug!(
                    peer = %remote_id,
                    layer = "iroh_transport",
                    error = ?err,
                    "failed to resolve inbound peer address"
                );
                continue;
            },
        };

        // Guarda para reaproveitamento futuro: se depois discarmos (open_bi) para este mesmo
        // peer+ALPN, reaproveitamos esta conexão em vez de abrir uma nova — QUIC é full-duplex,
        // então não importa que tenha sido o peer quem iniciou.
        connections.write().await.insert((peer.clone(), alpn.clone()), conn.clone());

        tracing::debug!(
            peer = %remote_id,
            layer = "iroh_transport",
            alpn = ?String::from_utf8_lossy(&alpn),
            "inbound connection established"
        );

        tokio::spawn(drive_incoming_streams(conn, peer, addr, alpn, incoming_tx.clone()));
    }
}

/// Aceita repetidamente novos streams bidirecionais nesta conexão específica, um por um,
/// enquanto ela estiver viva, encaminhando cada um para o `NetworkManager` via o canal que
/// alimenta `IrohTransport::accept`. Termina quando a conexão fecha (peer caiu, idle timeout,
/// ou nosso lado desligou) ou quando ninguém mais está lendo do canal (`NetworkManager` parado).
async fn drive_incoming_streams(
    conn: Connection, peer: PeerId, addr: PeerAddr, alpn: Vec<u8>,
    incoming_tx: mpsc::UnboundedSender<Box<dyn IncomingConnection>>,
) {
    let shared_conn = Arc::new(conn);

    loop {
        match shared_conn.accept_bi().await {
            Ok((send, recv)) => {
                let item: Box<dyn IncomingConnection> = Box::new(IrohIncoming::new(
                    send,
                    recv,
                    Arc::clone(&shared_conn),
                    peer.clone(),
                    addr.clone(),
                    alpn.clone(),
                ));

                if incoming_tx.send(item).is_err() {
                    break;
                }
            },
            Err(err) => {
                tracing::debug!(
                    peer = %peer.id,
                    layer = "iroh_transport",
                    error = ?err,
                    "connection closed, stopping stream acceptor"
                );
                break;
            },
        }
    }
}

/// Utilitário interno para compor o EndpointAddr a partir das informações de endereço de entrada.
fn resolve_endpoint_addr(remote_id: EndpointId, incoming_addr: &IncomingAddr) -> EndpointAddr {
    let mut endpoint_addr = EndpointAddr::new(remote_id);

    match incoming_addr {
        IncomingAddr::Ip(socket_address) => {
            endpoint_addr = endpoint_addr.with_ip_addr(*socket_address);
        },
        IncomingAddr::Relay { url: relay_url, .. } => {
            endpoint_addr = endpoint_addr.with_relay_url(relay_url.clone());
        },
        _ => {},
    }

    endpoint_addr
}

/// `true` só quando a tentativa de conexão de entrada chegou por um endereço IP direto de rede
/// local — qualquer outra via (relay, ou uma variante desconhecida do iroh) é tratada como não
/// confiável quando o modo de relay está desligado, já que nenhuma delas deveria sequer existir
/// nesse cenário (ver `restrict_to_lan` em `IrohTransport`).
fn is_private_incoming(incoming_addr: &IncomingAddr) -> bool {
    match incoming_addr {
        IncomingAddr::Ip(socket_address) => is_private_ip(socket_address.ip()),
        _ => false,
    }
}

/// Filtra um `EndpointAddr` cacheado pra manter só endereços IP de rede local, descartando
/// qualquer hint de relay — usado por `open_bi` quando não há relay configurado, pra um endereço
/// direto salvo de ANTES do usuário desligar o relay (ou um IP público que já tenha sido válido
/// em algum momento) não continuar permitindo dial cross-internet.
fn restrict_to_lan_addr(addr: EndpointAddr) -> EndpointAddr {
    let mut restricted = EndpointAddr::new(addr.id);
    for socket_address in addr.ip_addrs().filter(|socket_address| is_private_ip(socket_address.ip())) {
        restricted = restricted.with_ip_addr(*socket_address);
    }
    restricted
}

/// Faixas de rede local/privada (IPv4 RFC 1918 + link-local, IPv6 ULA/link-local/loopback) — o
/// resto (qualquer IP roteável publicamente) é tratado como WAN.
fn is_private_ip(ip: std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => v4.is_private() || v4.is_loopback() || v4.is_link_local(),
        std::net::IpAddr::V6(v6) => {
            v6.is_loopback()
                || (v6.segments()[0] & 0xfe00) == 0xfc00 // unique local: fc00::/7
                || (v6.segments()[0] & 0xffc0) == 0xfe80 // link-local: fe80::/10
        },
    }
}

#[cfg(test)]
mod tests {
    use tokio::io::AsyncWriteExt;

    use super::*;
    use crate::core::transport::{iroh::IrohTransportBuilder, TransportP2pBuilder};

    async fn build_transport() -> IrohTransport {
        IrohTransportBuilder::default().build(vec![b"test/proto".to_vec()]).await.unwrap()
    }

    #[tokio::test]
    async fn local_id_not_empty() {
        let transport = build_transport().await;
        assert!(!transport.local_id().id.is_empty());
    }

    #[tokio::test]
    async fn local_id_has_populated_device_id() {
        let transport = build_transport().await;
        assert!(transport.local_id().device_id.is_some());
    }

    #[tokio::test]
    async fn same_seed_generates_same_device_id() {
        let seed = [0x42u8; 32];
        let t1 = IrohTransportBuilder::default().seed(seed).build(vec![]).await.unwrap();
        let t2 = IrohTransportBuilder::default().seed(seed).build(vec![]).await.unwrap();
        assert_eq!(t1.local_id().device_id, t2.local_id().device_id);
    }

    #[tokio::test]
    async fn different_seeds_generate_different_device_ids() {
        let t1 = IrohTransportBuilder::default().seed([0x11u8; 32]).build(vec![]).await.unwrap();
        let t2 = IrohTransportBuilder::default().seed([0x22u8; 32]).build(vec![]).await.unwrap();
        assert_ne!(t1.local_id().device_id, t2.local_id().device_id);
    }

    #[tokio::test]
    async fn shutdown_without_error() {
        let transport = build_transport().await;
        assert!(transport.shutdown().await.is_ok());
        assert!(transport.endpoint.is_closed());
    }

    #[tokio::test]
    async fn iroh_transport_latency_and_flush_and_shutdown_integration() {
        let transport_a = Arc::new(build_transport().await);
        let transport_b = Arc::new(build_transport().await);

        let addr_b = transport_b.local_addr().unwrap();
        let peer_b_id = transport_b.local_id();

        let t_b = Arc::clone(&transport_b);
        let accept_handle = tokio::spawn(async move {
            let incoming = t_b.accept().await.unwrap();
            let (_in_writer, mut in_reader) = incoming.accept_bi().await.unwrap();
            let mut buf = [0u8; 4];
            let _ = tokio::io::AsyncReadExt::read_exact(&mut in_reader, &mut buf).await;
        });

        let (mut writer, _reader) = transport_a.open_bi(b"test/proto", &addr_b).await.unwrap();

        // Escreve e descarrega dados no ConnectionWriter ativo (exercita poll_flush)
        writer.write_all(b"ping").await.unwrap();
        writer.flush().await.unwrap();

        accept_handle.await.unwrap();

        // Checagem de latência: deve retornar Some(duration) quando conectado
        let mut latency_inherent = None;
        for _ in 0..30 {
            if let Some(l) = transport_a.latency(&peer_b_id).await {
                latency_inherent = Some(l);
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }

        let mut latency_trait = None;
        for _ in 0..30 {
            if let Some(l) = P2pTransport::latency(&*transport_a, &peer_b_id).await {
                latency_trait = Some(l);
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }

        assert!(latency_inherent.is_some(), "Inherent latency should return Some(Duration)");
        assert!(latency_trait.is_some(), "Trait latency should return Some(Duration)");

        // Encerra ambos os transportes
        transport_a.shutdown().await.unwrap();
        transport_b.shutdown().await.unwrap();
        assert!(transport_a.endpoint.is_closed());
        assert!(transport_b.endpoint.is_closed());
    }

    #[test]
    fn resolve_endpoint_addr_handles_relay_url() {
        // Verifica que o branch IncomingAddr::Relay associa a URL do Relay ao EndpointAddr
        let node_id = iroh::SecretKey::generate().public();
        let relay_url: iroh::RelayUrl =
            "https://relay.example.com.".parse().expect("Valid relay URL");
        let incoming_relay_address =
            IncomingAddr::Relay { url: relay_url.clone(), endpoint_id: node_id };

        let resolved_endpoint_address = resolve_endpoint_addr(node_id, &incoming_relay_address);

        assert!(resolved_endpoint_address.relay_urls().any(|u| u == &relay_url));
        assert_eq!(resolved_endpoint_address.id, node_id);
    }

    #[test]
    fn resolve_endpoint_addr_handles_ip_address() {
        // Verifica que o branch IncomingAddr::Ip associa o SocketAddr ao EndpointAddr
        let node_id = iroh::SecretKey::generate().public();
        let socket_address: std::net::SocketAddr =
            "127.0.0.1:8080".parse().expect("Valid SocketAddr");
        let incoming_ip_address = IncomingAddr::Ip(socket_address);

        let resolved_endpoint_address = resolve_endpoint_addr(node_id, &incoming_ip_address);

        assert_eq!(resolved_endpoint_address.id, node_id);
        assert!(resolved_endpoint_address.ip_addrs().any(|addr| *addr == socket_address));
    }

    #[test]
    fn is_private_ip_accepts_rfc1918_loopback_and_link_local() {
        assert!(is_private_ip("192.168.1.10".parse().unwrap()));
        assert!(is_private_ip("10.0.0.1".parse().unwrap()));
        assert!(is_private_ip("172.16.5.4".parse().unwrap()));
        assert!(is_private_ip("127.0.0.1".parse().unwrap()));
        assert!(is_private_ip("169.254.1.1".parse().unwrap()));
        assert!(is_private_ip("::1".parse().unwrap()));
        assert!(is_private_ip("fc00::1".parse().unwrap()));
        assert!(is_private_ip("fe80::1".parse().unwrap()));
    }

    #[test]
    fn is_private_ip_rejects_public_addresses() {
        assert!(!is_private_ip("8.8.8.8".parse().unwrap()));
        assert!(!is_private_ip("1.1.1.1".parse().unwrap()));
        assert!(!is_private_ip("2606:4700:4700::1111".parse().unwrap()));
    }

    #[test]
    fn is_private_incoming_true_only_for_private_ip() {
        let node_id = iroh::SecretKey::generate().public();
        assert!(is_private_incoming(&IncomingAddr::Ip("192.168.1.5:1234".parse().unwrap())));
        assert!(!is_private_incoming(&IncomingAddr::Ip("8.8.8.8:1234".parse().unwrap())));
        assert!(!is_private_incoming(&IncomingAddr::Relay {
            url: "https://relay.example.com.".parse().unwrap(),
            endpoint_id: node_id,
        }));
    }

    #[test]
    fn restrict_to_lan_addr_keeps_only_private_ips_and_drops_relay_hints() {
        let node_id = iroh::SecretKey::generate().public();
        let relay_url: iroh::RelayUrl = "https://relay.example.com.".parse().unwrap();

        let addr = EndpointAddr::new(node_id)
            .with_ip_addr("192.168.1.5:1234".parse().unwrap())
            .with_ip_addr("8.8.8.8:1234".parse().unwrap())
            .with_relay_url(relay_url.clone());

        let restricted = restrict_to_lan_addr(addr);

        assert_eq!(restricted.id, node_id);
        assert!(restricted.ip_addrs().any(|a| a.ip().to_string() == "192.168.1.5"));
        assert!(!restricted.ip_addrs().any(|a| a.ip().to_string() == "8.8.8.8"));
        assert_eq!(restricted.relay_urls().count(), 0);
    }

    #[test]
    fn restrict_to_lan_addr_empties_out_when_only_public_ip_cached() {
        let node_id = iroh::SecretKey::generate().public();
        let addr = EndpointAddr::new(node_id).with_ip_addr("8.8.8.8:1234".parse().unwrap());

        let restricted = restrict_to_lan_addr(addr);

        assert_eq!(restricted.ip_addrs().count(), 0);
    }

    #[tokio::test]
    async fn iroh_transport_latency_returns_none_for_unknown_peer() {
        let transport = build_transport().await;
        let unknown_peer = PeerId { id: "unknown-peer-id".to_string(), device_id: None };
        assert!(transport.latency(&unknown_peer).await.is_none());
        assert!(P2pTransport::latency(&transport, &unknown_peer).await.is_none());
    }

    /// Regressão: uma tentativa anterior desta correção guardava só um handle *fraco* na cache,
    /// o que fazia a conexão fechar (`implicit_close`) assim que as últimas referências fortes
    /// locais (writer/reader) saíam de escopo — exatamente o momento em que `handler.handle()`
    /// retorna em `NetworkManager`. Isso corria contra o outro lado, que podia ainda estar no
    /// meio de `accept_bi()`. Provado empiricamente: essa versão do teste falhava 5/5 vezes com
    /// handle fraco (`PeerDisconnected("connection closed by application")` no lado que aceita)
    /// e passava 3/3 com um clone forte independente por conexão. O pool de conexões
    /// reaproveitadas mantém esse clone forte deliberadamente vivo além do escopo de qualquer
    /// stream individual, então o outro lado sempre tem tempo de terminar sem corrida.
    #[tokio::test]
    async fn scoped_write_and_drop_does_not_race_receivers_accept_bi() {
        for _ in 0..5 {
            let transport_a = Arc::new(build_transport().await);
            let transport_b = Arc::new(build_transport().await);

            let addr_b = transport_b.local_addr().unwrap();

            let t_b = Arc::clone(&transport_b);
            let accept_handle = tokio::spawn(async move {
                let incoming = t_b.accept().await.unwrap();
                let (mut out_writer, mut in_reader) = incoming.accept_bi().await.unwrap();
                let mut buf = [0u8; 4];
                tokio::io::AsyncReadExt::read_exact(&mut in_reader, &mut buf).await.unwrap();
                out_writer.shutdown().await.unwrap();
            });

            {
                let (mut writer, _reader) =
                    transport_a.open_bi(b"test/proto", &addr_b).await.unwrap();
                writer.write_all(b"ping").await.unwrap();
                writer.flush().await.unwrap();
                writer.shutdown().await.unwrap();
                // `writer`/`_reader` somem no fim deste escopo — é exatamente o que acontece
                // hoje quando `handler.handle()` retorna em `NetworkManager`, bem antes de
                // sabermos se o outro lado já terminou de ler.
            }

            accept_handle.await.unwrap();

            transport_a.shutdown().await.unwrap();
            transport_b.shutdown().await.unwrap();
        }
    }

    /// Regressão do bug relatado: sem relay configurado (`build_transport` usa o builder
    /// default, que resolve pra `RelayMode::Disabled`), um `PeerAddr` cacheado de ANTES do relay
    /// ser desligado (ou simplesmente forjado) cujo único endereço é um IP público não pode
    /// continuar valendo pro dial — prova que `open_bi` de fato aplica `restrict_to_lan_addr`
    /// antes de discar (não só que a função pura filtra certo, já coberto acima), inspecionando
    /// o `EndpointAddr` que `dial_or_reuse` recebeu via o mesmo parse que `open_bi` faria.
    #[test]
    fn open_bi_would_strip_public_ip_only_cached_address_when_relay_disabled() {
        let node_id = iroh::SecretKey::generate().public();
        let cached = EndpointAddr::new(node_id).with_ip_addr("203.0.113.10:1234".parse().unwrap());
        let addrs = serde_json::to_vec(&cached).unwrap();

        let decoded: EndpointAddr = serde_json::from_slice(&addrs).unwrap();
        let restricted = restrict_to_lan_addr(decoded);

        assert_eq!(restricted.ip_addrs().count(), 0, "public IP must not survive the LAN filter");
        assert_eq!(restricted.relay_urls().count(), 0);
    }

    /// O ganho principal do pool: evita discar uma conexão QUIC nova a cada `open_bi` para o
    /// mesmo peer+ALPN — que é o que causava as colisões de multipath/handshake observadas em
    /// produção (`MultipathNotNegotiated`, `handshake failed`) quando duas sessões discavam para
    /// o mesmo peer em sequência rápida.
    #[tokio::test]
    async fn open_bi_reuses_cached_connection_across_calls() {
        let transport_a = Arc::new(build_transport().await);
        let transport_b = Arc::new(build_transport().await);

        let addr_b = transport_b.local_addr().unwrap();

        async fn round_trip(
            transport_a: &Arc<IrohTransport>, transport_b: &Arc<IrohTransport>, addr_b: &PeerAddr,
        ) {
            let t_b = Arc::clone(transport_b);
            let accept_handle = tokio::spawn(async move {
                let incoming = t_b.accept().await.unwrap();
                let (_out_writer, mut in_reader) = incoming.accept_bi().await.unwrap();
                let mut buf = [0u8; 4];
                let _ = tokio::io::AsyncReadExt::read_exact(&mut in_reader, &mut buf).await;
            });

            let (mut writer, _reader) = transport_a.open_bi(b"test/proto", addr_b).await.unwrap();
            writer.write_all(b"ping").await.unwrap();
            writer.flush().await.unwrap();

            accept_handle.await.unwrap();
        }

        round_trip(&transport_a, &transport_b, &addr_b).await;

        let key = (addr_b.id.clone(), b"test/proto".to_vec());
        let first_stable_id =
            transport_a.connections.read().await.get(&key).map(|conn| conn.stable_id());
        assert!(first_stable_id.is_some(), "connection should be cached after the first open_bi");

        round_trip(&transport_a, &transport_b, &addr_b).await;

        let second_stable_id =
            transport_a.connections.read().await.get(&key).map(|conn| conn.stable_id());

        assert_eq!(
            first_stable_id, second_stable_id,
            "second open_bi to the same peer+ALPN should reuse the cached connection, not dial \
             a fresh one"
        );

        transport_a.shutdown().await.unwrap();
        transport_b.shutdown().await.unwrap();
    }

    /// Regressão da race em `open_bi`/`dial_or_reuse`: sem lock por chave, o trecho
    /// `check cache → dial → insert cache` não é atômico — entre o `read().await` e o
    /// `write().await.insert(...)` existe um `.await` inteiro (`endpoint.connect`) sem nenhum
    /// lock segurando a chave, então chamadas concorrentes para o mesmo `(peer, alpn)` podiam
    /// todas ver o cache vazio e discar conexões físicas duplicadas. Dispara `N` chamadas
    /// concorrentes de `dial_or_reuse` para a mesma chave (via `tokio::spawn`, não sequenciais)
    /// e confere que todas resolvem para a mesma conexão física (`stable_id` idêntico) — se a
    /// race reaparecer, cada chamada perdedora do lock discaria sua própria conexão e os
    /// `stable_id`s divergiriam.
    #[tokio::test]
    async fn dial_or_reuse_serializes_concurrent_dials_for_same_key() {
        let transport_a = Arc::new(build_transport().await);
        let transport_b = Arc::new(build_transport().await);

        let addr_b = transport_b.local_addr().unwrap();
        let addr: EndpointAddr = serde_json::from_slice(&addr_b.addrs).unwrap();
        let key: ConnectionKey = (addr_b.id.clone(), b"test/proto".to_vec());

        const N: usize = 8;

        let handles: Vec<_> = (0..N)
            .map(|_| {
                let transport_a = Arc::clone(&transport_a);
                let addr = addr.clone();
                let key = key.clone();
                tokio::spawn(async move {
                    transport_a
                        .dial_or_reuse(&key, &addr, b"test/proto", &key.0)
                        .await
                        .unwrap()
                        .stable_id()
                })
            })
            .collect();

        let mut stable_ids = Vec::with_capacity(N);
        for handle in handles {
            stable_ids.push(handle.await.unwrap());
        }

        let first = stable_ids[0];
        assert!(
            stable_ids.iter().all(|id| *id == first),
            "all concurrent dials for the same (peer, alpn) must resolve to a single physical \
             connection, got {stable_ids:?}"
        );

        transport_a.shutdown().await.unwrap();
        transport_b.shutdown().await.unwrap();
    }

    /// `is_connected` é a base da visão "peer online agora": deve refletir o cache de conexões
    /// físicas vivas, não nenhuma sessão de protocolo de aplicação (que pode já ter terminado).
    #[tokio::test]
    async fn is_connected_reflects_live_physical_connection() {
        let transport_a = Arc::new(build_transport().await);
        let transport_b = Arc::new(build_transport().await);

        let addr_b = transport_b.local_addr().unwrap();
        let peer_b_id = transport_b.local_id();

        let t_b = Arc::clone(&transport_b);
        let accept_handle = tokio::spawn(async move {
            let incoming = t_b.accept().await.unwrap();
            let (_out_writer, mut in_reader) = incoming.accept_bi().await.unwrap();
            let mut buf = [0u8; 4];
            let _ = tokio::io::AsyncReadExt::read_exact(&mut in_reader, &mut buf).await;
        });

        let (mut writer, _reader) = transport_a.open_bi(b"test/proto", &addr_b).await.unwrap();
        writer.write_all(b"ping").await.unwrap();
        writer.flush().await.unwrap();
        accept_handle.await.unwrap();

        assert!(
            P2pTransport::is_connected(&*transport_a, &peer_b_id).await,
            "dialing side should see a live connection right after open_bi"
        );

        transport_a.shutdown().await.unwrap();
        transport_b.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn is_connected_returns_false_for_unknown_peer() {
        let transport = build_transport().await;
        let unknown_peer = PeerId { id: "unknown-peer-id".to_string(), device_id: None };
        assert!(!P2pTransport::is_connected(&transport, &unknown_peer).await);
    }

    /// Regressão do bug "timed out reading library summary" persistindo até reiniciar o app no
    /// Android: uma conexão reaproveitada pode estar silenciosamente morta (path QUIC caído sem
    /// `close_reason()` ainda refletir isso) e um handler de protocolo detecta isso via timeout
    /// de aplicação, bem antes do idle timeout do QUIC expirar. Sem `invalidate`, toda tentativa
    /// seguinte reaproveitaria essa mesma conexão morta e falharia do mesmo jeito
    /// indefinidamente. Prova que `invalidate` remove a entrada do pool, forçando o próximo
    /// `open_bi` a discar uma conexão física nova em vez de reaproveitar a antiga.
    #[tokio::test]
    async fn invalidate_forces_fresh_dial_on_next_open_bi() {
        let transport_a = Arc::new(build_transport().await);
        let transport_b = Arc::new(build_transport().await);

        let addr_b = transport_b.local_addr().unwrap();

        async fn round_trip(
            transport_a: &Arc<IrohTransport>, transport_b: &Arc<IrohTransport>, addr_b: &PeerAddr,
        ) {
            let t_b = Arc::clone(transport_b);
            let accept_handle = tokio::spawn(async move {
                let incoming = t_b.accept().await.unwrap();
                let (_out_writer, mut in_reader) = incoming.accept_bi().await.unwrap();
                let mut buf = [0u8; 4];
                let _ = tokio::io::AsyncReadExt::read_exact(&mut in_reader, &mut buf).await;
            });

            let (mut writer, _reader) = transport_a.open_bi(b"test/proto", addr_b).await.unwrap();
            writer.write_all(b"ping").await.unwrap();
            writer.flush().await.unwrap();

            accept_handle.await.unwrap();
        }

        round_trip(&transport_a, &transport_b, &addr_b).await;

        let key = (addr_b.id.clone(), b"test/proto".to_vec());
        let first_stable_id =
            transport_a.connections.read().await.get(&key).map(|conn| conn.stable_id());
        assert!(first_stable_id.is_some(), "connection should be cached after the first open_bi");

        P2pTransport::invalidate(&*transport_a, &addr_b.id, b"test/proto").await;
        assert!(
            transport_a.connections.read().await.get(&key).is_none(),
            "invalidate should remove the cached entry"
        );

        round_trip(&transport_a, &transport_b, &addr_b).await;

        let second_stable_id =
            transport_a.connections.read().await.get(&key).map(|conn| conn.stable_id());
        assert!(second_stable_id.is_some(), "connection should be cached again after redial");
        assert_ne!(
            first_stable_id, second_stable_id,
            "after invalidate, open_bi must dial a fresh physical connection, not reuse the old one"
        );

        transport_a.shutdown().await.unwrap();
        transport_b.shutdown().await.unwrap();
    }

    /// `network_change` só repassa para `Endpoint::network_change()` — no Android é essa a única
    /// forma de o transporte saber que a rede mudou, já que o SO não expõe monitoramento de
    /// interface pra código nativo (só via `ConnectivityManager` do lado Kotlin). Prova só que a
    /// chamada não entra em erro/panic contra um endpoint vivo.
    #[tokio::test]
    async fn network_change_does_not_panic_on_live_endpoint() {
        let transport = build_transport().await;
        P2pTransport::network_change(&transport).await;
        transport.shutdown().await.unwrap();
    }
}
