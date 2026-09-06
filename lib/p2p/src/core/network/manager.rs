//! Gestão de ciclos de execução, roteamento e controle central da rede.
//!
//! O `NetworkManager` atua como o cérebro assíncrono da biblioteca.
//! Ele encapsula a instância de transporte físico (ex: Iroh), executa o laço de eventos
//! (event loop) para aceitar conexões ativamente, e faz a ponte (dispatch) entre os
//! canais I/O recém-chegados e o respectivo `ProtocolHandler` mapeado para o ALPN requisitado.

use std::{collections::HashMap, sync::Arc};

use tokio::sync::{mpsc, RwLock};
use tracing::Instrument;

use crate::{
    core::{
        guard::{BoxedValidator, ConnectionContext},
        network::state::{NetworkMode, NetworkState},
        storage::P2PStorage,
        transport::P2pTransport,
    },
    data::protocol::{EventEmitter, ProtocolHandler},
    infra::{
        error::ConnectionError,
        peer::{PeerAddr, PeerId},
    },
};

/// Limite de comandos simultâneos não processados na fila do loop principal.
const COMMAND_CHANNEL_CAPACITY: usize = 64;

/// Teto por tentativa individual de discagem outbound em `handle_connect_command`.
///
/// Sem isso, cada uma das 5 tentativas de retry esperava o `endpoint.connect()` do iroh
/// inteiro resolver sozinho — o que pode levar perto de 10s quando o endereço cacheado do
/// peer está morto (peer trocou de rede/Wi-Fi) antes do relay assumir. 5 tentativas
/// sequenciais nesse cenário significam até ~50s de espera percebida pelo usuário só pra
/// descobrir que o endereço direto está morto, quando o relay (se disponível) normalmente
/// resolve bem mais rápido que isso. Um teto curto por tentativa faz a malha de retry
/// desistir do endereço morto e cair pro relay bem mais cedo, sem esperar o timeout interno
/// completo do iroh em cada uma das 5 rodadas.
const CONNECT_ATTEMPT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(3);

/// Sinais de controle enviados ao Event Loop da rede.
///
/// Como o manager é blindado e opera numa tarefa em background, toda interação
/// externa é sinalizada e enfileirada no canal por meio deste enum.
pub enum NetworkCommand {
    /// Troca dinâmica da política de validação (Guard) e estado nominal da rede.
    SwitchGuard {
        /// O novo validador Guard a ser aplicado.
        validator: BoxedValidator,
        /// O novo modo de operação da rede.
        mode: NetworkMode,
    },
    /// Tenta discar ativamente para outro par através de um protocolo.
    Connect {
        /// Endereço de destino do peer.
        addr: PeerAddr,
        /// Protocolo ALPN requisitado.
        alpn: Vec<u8>,
    },
    /// Provoca a desmontagem e desligamento seguro do daemon P2P.
    Shutdown,
}

/// Motor principal do nó acerola-p2p, responsável pelo event loop e orquestração.
pub struct NetworkManager {
    /// Provedor base responsável por I/O e alocação de sockets (Iroh).
    transport: Arc<dyn P2pTransport>,
    /// Referência concorrente para o estado (peers conectados, etc).
    state: Arc<RwLock<NetworkState>>,
    /// Referência do Guard atual ativo para validação no aceite de conexões.
    validator: Arc<RwLock<BoxedValidator>>,
    /// Componente de persistência de identidade e peers (P2PStorage).
    storage: Option<Arc<dyn P2PStorage>>,
    /// Fila para consumo dos comandos requisitados externamente.
    command_rx: mpsc::Receiver<NetworkCommand>,
    /// Tabela de protocolos autorizados para quem recebe conexões (Servidor).
    handlers_inbound: HashMap<Vec<u8>, Arc<dyn ProtocolHandler>>,
    /// Tabela de protocolos operados por quem inicia conexões (Cliente).
    handlers_outbound: HashMap<Vec<u8>, Arc<dyn ProtocolHandler>>,
    /// Canal de emissão de eventos assíncronos para a camada de aplicação/UI.
    emit: EventEmitter,
}

impl NetworkManager {
    /// Inicializa os componentes internos de gerência de rede.
    ///
    /// Cria e compartilha buffers MPSC e o `NetworkState`. Retorna uma tupla
    /// contendo a instância pronta para rodar, o comunicador (sender) e a view
    /// do estado da rede, garantindo que o chamador mantenha as referências ativas.
    #[allow(dead_code)]
    pub fn new(
        transport: Arc<dyn P2pTransport>, validator: BoxedValidator, emit: EventEmitter,
    ) -> (Self, mpsc::Sender<NetworkCommand>, Arc<RwLock<NetworkState>>) {
        Self::with_storage(transport, validator, emit, None)
    }

    /// Inicializa os componentes internos de gerência de rede com suporte opcional a storage.
    pub fn with_storage(
        transport: Arc<dyn P2pTransport>, validator: BoxedValidator, emit: EventEmitter,
        storage: Option<Arc<dyn P2PStorage>>,
    ) -> (Self, mpsc::Sender<NetworkCommand>, Arc<RwLock<NetworkState>>) {
        let (command_tx, command_rx) = mpsc::channel(COMMAND_CHANNEL_CAPACITY);
        let state = Arc::new(RwLock::new(NetworkState::new()));

        let manager = Self {
            transport,
            command_rx,
            state: Arc::clone(&state),
            handlers_inbound: HashMap::new(),
            handlers_outbound: HashMap::new(),
            validator: Arc::new(RwLock::new(validator)),
            storage,
            emit,
        };

        (manager, command_tx, state)
    }

    /// Registra um serviço voltado ao recebimento passivo de conexões.
    ///
    /// Mapeia uma string ALPN a um `ProtocolHandler`. Ao receber conexões com este ALPN,
    /// a conexão passará primeiro pelo guard e, se permitida, será roteada a este tratador.
    pub fn register_inbound(&mut self, alpn: &[u8], handler: Arc<dyn ProtocolHandler>) {
        self.handlers_inbound.insert(alpn.to_vec(), handler);
    }

    /// Registra um serviço focado no disparo ativo de conexões ao ecossistema.
    ///
    /// Mapeia o ALPN em cenários em que o nó é proativo (dialer) em invocar funcionalidades.
    pub fn register_outbound(&mut self, alpn: &[u8], handler: Arc<dyn ProtocolHandler>) {
        self.handlers_outbound.insert(alpn.to_vec(), handler);
    }

    /// Assume o controle da Thread e dispara o laço principal de IO da biblioteca.
    ///
    /// Deve ser convocado via `tokio::spawn(manager.run())`. Durante este loop infinito
    /// ele espera assincronamente (multiplexando pelo `tokio::select!`) por conexões
    /// externas advindas do Transporte e comandos oriundos dos canais MPSC locais.
    pub async fn run(mut self) {
        let mut latency_interval = tokio::time::interval(std::time::Duration::from_secs(30));
        latency_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = latency_interval.tick() => {
                    self.emit_latency_for_all_peers().await;
                }
                result = self.transport.accept() => {
                    match result {
                        Ok(incoming) => self.handle_incoming(incoming),
                        Err(ConnectionError::Shutdown) => break,
                        Err(err) => {
                            tracing::debug!(error = ?err, "transport accept failed");
                        }
                    }
                }
                cmd_option = self.command_rx.recv() => {
                    match cmd_option {
                        Some(NetworkCommand::Connect { addr, alpn }) => {
                            self.handle_connect_command(addr, alpn);
                        }
                        Some(NetworkCommand::SwitchGuard { validator, mode }) => {
                            self.handle_switch_guard(validator, mode).await;
                        }
                        Some(NetworkCommand::Shutdown) => {
                            // Antes, este branch só saía do loop sem nunca desligar o
                            // transporte por baixo — o `Endpoint` real do iroh (`IrohTransport`)
                            // continuava vivo e escutando pra sempre, e a task separada de
                            // accept-loop (`drive_incoming_connections`) também, porque nada
                            // nunca chamava `endpoint.close()`. Um "reiniciar" que fizesse
                            // shutdown+rebuild vazaria o socket UDP antigo. `transport.shutdown()`
                            // fecha o endpoint de verdade; o próprio `drive_incoming_connections`
                            // já sai sozinho quando isso acontece (`endpoint.accept()` retorna
                            // `None`), sem precisar de nenhuma outra sinalização.
                            if let Err(error) = self.transport.shutdown().await {
                                tracing::warn!(?error, "failed to shut down transport cleanly");
                            }
                            break;
                        }
                        None => {
                            // O canal command_tx foi fechado (ex: a instância AcerolaP2p foi dropada sem shutdown explícito)
                            break;
                        }
                    }
                }
            }
        }
    }

    /// Emite eventos de latência para todos os peers atualmente conectados.
    async fn emit_latency_for_all_peers(&self) {
        let peers: Vec<PeerId> = self.state.read().await.peers().keys().cloned().collect();

        for peer in peers {
            if let Some(latency) = self.transport.latency(&peer).await {
                let payload = serde_json::json!({
                    "peer_id": peer.id,
                    "latency_ms": latency.as_millis() as u64,
                })
                .to_string();
                (self.emit)("network:latency", payload);
            }
        }
    }

    /// Aceita e despacha uma conexão inbound em background.
    ///
    /// Ignora silenciosamente conexões cujo ALPN não possui handler registrado.
    fn handle_incoming(&self, incoming: Box<dyn crate::core::transport::IncomingConnection>) {
        let Some(handler) = self.handlers_inbound.get(incoming.alpn()) else { return };

        let state = Arc::clone(&self.state);
        let handler = handler.clone();
        let validator = Arc::clone(&self.validator);
        let storage = self.storage.clone();

        let span = tracing::info_span!(
            "inbound",
            peer = %incoming.peer().id,
            alpn = ?String::from_utf8_lossy(incoming.alpn())
        );

        tokio::spawn(
            async move {
                let peer = incoming.peer().clone();
                let addr = incoming.addr().clone();
                let alpn = incoming.alpn().to_vec();

                let Ok((send, recv)) = incoming.accept_bi().await else { return };

                let ctx = ConnectionContext { peer_id: peer.clone(), data: () };
                let allowed = {
                    let guard = validator.read().await;
                    guard(&ctx)
                }
                .await;

                if let Err(err) = allowed {
                    tracing::debug!(error = ?err, "connection denied by guard");
                    return;
                }

                if save_peer_if_present(storage.as_ref(), &addr).await.is_err() {
                    tracing::error!(peer = %peer.id, "failed to save peer to storage, terminating connection");
                    return;
                }

                state.write().await.connect(peer.clone(), addr, alpn.clone());
                tracing::debug!("connection accepted");

                if let Err(err) = handler.handle(&peer, send, recv).await {
                    tracing::warn!(error = ?err, "inbound handler failed");
                }
                tracing::debug!("connection closed");

                state.write().await.disconnect(&peer, &alpn);
            }
            .instrument(span),
        );
    }

    /// Dispara uma tentativa de conexão outbound com retries em background.
    ///
    /// Ignora silenciosamente se não há handler registrado para o ALPN solicitado.
    fn handle_connect_command(&self, addr: PeerAddr, alpn: Vec<u8>) {
        let Some(handler) = self.handlers_outbound.get(&alpn) else { return };

        let state = Arc::clone(&self.state);
        let handler = handler.clone();
        let transport = Arc::clone(&self.transport);
        let storage = self.storage.clone();

        let span = tracing::info_span!(
            "outbound",
            addr = %addr.id,
            alpn = ?String::from_utf8_lossy(&alpn)
        );

        tokio::spawn(
            async move {
                let max_retries = 5;
                let mut backoff = std::time::Duration::from_millis(100);

                for attempt in 1..=max_retries {
                    let dial_result =
                        match tokio::time::timeout(CONNECT_ATTEMPT_TIMEOUT, transport.open_bi(&alpn, &addr))
                            .await
                        {
                            Ok(result) => result,
                            Err(_elapsed) => Err(ConnectionError::Timeout),
                        };

                    match dial_result {
                        Ok((send, recv)) => {
                            if save_peer_if_present(storage.as_ref(), &addr).await.is_err() {
                                tracing::error!(peer = %addr.id, "failed to save outbound peer to storage, terminating connection");
                                return;
                            }

                            state.write().await.connect(addr.id.clone(), addr.clone(), alpn.clone());
                            tracing::debug!("outbound connection established");

                            if let Err(err) = handler.handle(&addr.id, send, recv).await {
                                tracing::warn!(error = ?err, "outbound handler failed");
                                // A conexão reaproveitada pode estar silenciosamente morta (ver
                                // doc em `P2pTransport::invalidate`) — descarta do pool pra
                                // próxima tentativa discar uma nova em vez de repetir a mesma
                                // falha indefinidamente.
                                transport.invalidate(&addr.id, &alpn).await;
                            }

                            tracing::debug!("outbound connection closed");
                            state.write().await.disconnect(&addr.id, &alpn);
                            return;
                        }
                        Err(err) => {
                            tracing::warn!(
                                attempt,
                                error = ?err,
                                "outbound connection attempt failed, retrying..."
                            );
                            if attempt < max_retries {
                                tokio::time::sleep(backoff).await;
                                backoff = (backoff * 2).min(std::time::Duration::from_secs(5));
                            }
                        }
                    }
                }
            }
            .instrument(span),
        );
    }

    /// Substitui on-the-fly o Guard ativo e o modo operacional da rede.
    async fn handle_switch_guard(&self, validator: BoxedValidator, mode: NetworkMode) {
        *self.validator.write().await = validator;
        self.state.write().await.switch_mode(mode);
    }
}

/// Utilitário funcional para persistir peer apenas se o storage estiver configurado.
async fn save_peer_if_present(
    storage: Option<&Arc<dyn P2PStorage>>, addr: &PeerAddr,
) -> Result<(), ConnectionError> {
    match storage {
        Some(storage) => storage.save_peer(addr).await,
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use tokio::time::{sleep, Duration};

    use super::*;
    use crate::{infra::peer::PeerId, tests::mock_transport::mock_transport};

    fn open_validator() -> BoxedValidator {
        Box::new(|_ctx| Box::pin(async { Ok(()) }))
    }

    fn no_op_emitter() -> EventEmitter {
        Arc::new(|_event: &str, _payload: String| {})
    }

    #[allow(clippy::type_complexity)]
    fn capture_emitter() -> (EventEmitter, Arc<Mutex<Vec<(String, String)>>>) {
        let events = Arc::new(Mutex::new(Vec::new()));
        let clone = Arc::clone(&events);
        let emit: EventEmitter = Arc::new(move |event: &str, payload: String| {
            clone.lock().unwrap().push((event.to_string(), payload));
        });
        (emit, events)
    }

    fn make_peer(id: &str) -> PeerId {
        PeerId { id: id.to_string(), device_id: None }
    }

    struct NoopHandler;
    #[async_trait::async_trait]
    impl ProtocolHandler for NoopHandler {
        async fn handle(
            &self, _peer: &PeerId, _send: Box<dyn tokio::io::AsyncWrite + Send + Unpin>,
            _recv: Box<dyn tokio::io::AsyncRead + Send + Unpin>,
        ) -> Result<(), ConnectionError> {
            Ok(())
        }
    }

    struct SlowHandler;
    #[async_trait::async_trait]
    impl ProtocolHandler for SlowHandler {
        async fn handle(
            &self, _peer: &PeerId, _send: Box<dyn tokio::io::AsyncWrite + Send + Unpin>,
            _recv: Box<dyn tokio::io::AsyncRead + Send + Unpin>,
        ) -> Result<(), ConnectionError> {
            sleep(Duration::from_millis(50)).await;
            Ok(())
        }
    }

    #[tokio::test]
    async fn inbound_handler_registered_for_alpn_is_found() {
        let (transport, _handle) = mock_transport();
        let (mut manager, _, _) =
            NetworkManager::new(Arc::new(transport), open_validator(), no_op_emitter());
        manager.register_inbound(b"acerola/handshake/1", Arc::new(NoopHandler));
        assert!(manager.handlers_inbound.contains_key(b"acerola/handshake/1".as_ref()));
    }

    #[tokio::test]
    async fn outbound_handler_registered_for_alpn_is_found() {
        let (transport, _handle) = mock_transport();
        let (mut manager, _, _) =
            NetworkManager::new(Arc::new(transport), open_validator(), no_op_emitter());
        manager.register_outbound(b"acerola/handshake/1", Arc::new(NoopHandler));
        assert!(manager.handlers_outbound.contains_key(b"acerola/handshake/1".as_ref()));
    }

    #[tokio::test]
    async fn peer_added_to_state_on_accepting_connection() {
        let (transport, handle) = mock_transport();
        let transport: Arc<dyn P2pTransport> = Arc::new(transport);
        let (mut manager, _command_tx, state) =
            NetworkManager::new(Arc::clone(&transport), open_validator(), no_op_emitter());
        manager.register_inbound(b"acerola/handshake/1", Arc::new(SlowHandler));

        let (client, server) = tokio::io::duplex(1024);
        handle.inject(b"acerola/handshake/1", make_peer("peer-1"), client, server);

        tokio::spawn(manager.run());
        sleep(Duration::from_millis(20)).await;

        assert!(state.read().await.is_connected(&make_peer("peer-1")));
    }

    #[tokio::test]
    async fn peer_removed_from_state_when_handler_finishes() {
        let (transport, handle) = mock_transport();
        let transport: Arc<dyn P2pTransport> = Arc::new(transport);
        let (mut manager, _command_tx, state) =
            NetworkManager::new(Arc::clone(&transport), open_validator(), no_op_emitter());
        manager.register_inbound(b"acerola/handshake/1", Arc::new(NoopHandler));

        let (client, server) = tokio::io::duplex(1024);
        handle.inject(b"acerola/handshake/1", make_peer("peer-2"), client, server);

        tokio::spawn(manager.run());
        sleep(Duration::from_millis(50)).await;

        assert!(!state.read().await.is_connected(&make_peer("peer-2")));
    }

    #[tokio::test]
    async fn unknown_alpn_is_ignored() {
        let (transport, handle) = mock_transport();
        let transport: Arc<dyn P2pTransport> = Arc::new(transport);
        let (manager, _command_tx, state) =
            NetworkManager::new(Arc::clone(&transport), open_validator(), no_op_emitter());

        let (client, server) = tokio::io::duplex(1024);
        handle.inject(b"acerola/unknown", make_peer("peer-3"), client, server);

        tokio::spawn(manager.run());
        sleep(Duration::from_millis(20)).await;

        assert!(!state.read().await.is_connected(&make_peer("peer-3")));
    }

    #[tokio::test]
    async fn shutdown_terminates_loop() {
        let (transport, _handle) = mock_transport();
        let (manager, command_tx, _) =
            NetworkManager::new(Arc::new(transport), open_validator(), no_op_emitter());

        let handle = tokio::spawn(manager.run());
        let _ = command_tx.send(NetworkCommand::Shutdown).await;

        let result = tokio::time::timeout(Duration::from_millis(100), handle).await;
        assert!(result.is_ok());
    }

    /// Regressão: `NetworkCommand::Shutdown` só saía do `select!` loop sem nunca chamar
    /// `transport.shutdown()` — em produção (`IrohTransport`) isso significava que o `Endpoint`
    /// real nunca fechava (`lib/p2p/TODO.md`), deixando o socket UDP e a task de accept-loop
    /// vivos pra sempre mesmo depois de um "shutdown" bem-sucedido. Prova que o transporte é
    /// desligado de verdade, não só abandonado.
    #[tokio::test]
    async fn shutdown_command_shuts_down_the_transport() {
        let (transport, transport_handle) = mock_transport();
        let (manager, command_tx, _) =
            NetworkManager::new(Arc::new(transport), open_validator(), no_op_emitter());

        let handle = tokio::spawn(manager.run());
        let _ = command_tx.send(NetworkCommand::Shutdown).await;

        let result = tokio::time::timeout(Duration::from_millis(100), handle).await;
        assert!(result.is_ok(), "shutdown should not hang");
        assert!(
            transport_handle.was_shutdown_called(),
            "NetworkCommand::Shutdown deveria chamar transport.shutdown()"
        );
    }

    #[tokio::test]
    async fn guard_denies_connection_from_blocked_peer() {
        let (transport, handle) = mock_transport();
        let transport: Arc<dyn P2pTransport> = Arc::new(transport);

        let deny_all: BoxedValidator = Box::new(|_ctx| {
            Box::pin(async { Err(ConnectionError::AuthDenied("test deny all".into())) })
        });

        let (mut manager, _command_tx, state) =
            NetworkManager::new(Arc::clone(&transport), deny_all, no_op_emitter());
        manager.register_inbound(b"acerola/handshake/1", Arc::new(SlowHandler));

        let (client, server) = tokio::io::duplex(1024);
        handle.inject(b"acerola/handshake/1", make_peer("peer-blocked"), client, server);

        tokio::spawn(manager.run());
        sleep(Duration::from_millis(30)).await;

        assert!(!state.read().await.is_connected(&make_peer("peer-blocked")));
    }

    #[tokio::test]
    async fn same_peer_on_two_alpns_appears_connected() {
        let (transport, handle) = mock_transport();
        let transport: Arc<dyn P2pTransport> = Arc::new(transport);
        let (mut manager, _command_tx, state) =
            NetworkManager::new(Arc::clone(&transport), open_validator(), no_op_emitter());

        manager.register_inbound(b"acerola/handshake/1", Arc::new(SlowHandler));
        manager.register_inbound(b"acerola/blob/1", Arc::new(SlowHandler));

        let (c1, s1) = tokio::io::duplex(1024);
        let (c2, s2) = tokio::io::duplex(1024);
        handle.inject(b"acerola/handshake/1", make_peer("peer-multi"), c1, s1);
        handle.inject(b"acerola/blob/1", make_peer("peer-multi"), c2, s2);

        tokio::spawn(manager.run());
        sleep(Duration::from_millis(20)).await;

        assert!(state.read().await.is_connected(&make_peer("peer-multi")));
        assert!(state
            .read()
            .await
            .is_connected_on(&make_peer("peer-multi"), b"acerola/handshake/1"));
        assert!(state.read().await.is_connected_on(&make_peer("peer-multi"), b"acerola/blob/1"));
    }

    #[tokio::test]
    async fn latency_event_emitted_for_connected_peers() {
        let (transport, handle) = mock_transport();
        let peer = make_peer("peer-latency");
        handle.set_latency(peer.clone(), Duration::from_millis(42)).await;

        let (emit, events) = capture_emitter();
        let transport: Arc<dyn P2pTransport> = Arc::new(transport);
        let (mut manager, _command_tx, _state) =
            NetworkManager::new(Arc::clone(&transport), open_validator(), emit);
        manager.register_inbound(b"acerola/handshake/1", Arc::new(SlowHandler));

        let (client, server) = tokio::io::duplex(1024);
        handle.inject(b"acerola/handshake/1", peer.clone(), client, server);

        tokio::spawn(manager.run());
        sleep(Duration::from_millis(50)).await;

        let captured = events.lock().unwrap();
        assert!(captured.iter().any(|(ev, payload)| {
            ev == "network:latency" && payload.contains("peer-latency") && payload.contains("42")
        }));
    }

    struct FailingStorage;
    #[async_trait::async_trait]
    impl P2PStorage for FailingStorage {
        async fn save_identity(&self, _secret: &[u8]) -> Result<(), ConnectionError> {
            Ok(())
        }
        async fn load_identity(&self) -> Result<Option<Vec<u8>>, ConnectionError> {
            Ok(None)
        }
        async fn save_peer(&self, _peer: &PeerAddr) -> Result<(), ConnectionError> {
            Err(ConnectionError::StreamFailed("disk write error".to_string()))
        }
        async fn load_peers(&self) -> Result<Vec<PeerAddr>, ConnectionError> {
            Ok(vec![])
        }
        async fn save_device_info(
            &self, _peer: &PeerId, _info: &crate::data::identity::device_info::DeviceInfo,
        ) -> Result<(), ConnectionError> {
            Ok(())
        }
        async fn load_device_info(
            &self,
        ) -> Result<Vec<(PeerId, crate::data::identity::device_info::DeviceInfo)>, ConnectionError>
        {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn inbound_connection_terminated_and_no_ghost_peer_when_save_peer_fails() {
        let (transport, handle) = mock_transport();
        let transport: Arc<dyn P2pTransport> = Arc::new(transport);
        let storage = Arc::new(FailingStorage);

        let (mut manager, _command_tx, state) = NetworkManager::with_storage(
            Arc::clone(&transport),
            open_validator(),
            no_op_emitter(),
            Some(storage),
        );
        manager.register_inbound(b"acerola/handshake/1", Arc::new(SlowHandler));

        let (client, server) = tokio::io::duplex(1024);
        handle.inject(b"acerola/handshake/1", make_peer("peer-storage-fail"), client, server);

        tokio::spawn(manager.run());
        sleep(Duration::from_millis(30)).await;

        assert!(!state.read().await.is_connected(&make_peer("peer-storage-fail")));
    }

    /// A liveness da conexão física é responsabilidade do keepalive nativo do QUIC/iroh, não de
    /// um sinal de GOODBYE de aplicação (removido — nunca disparava de verdade em produção, e
    /// mesmo quando disparava era rejeitado pelo lado receptor por pular a etapa de handshake).
    /// O shutdown deve terminar o loop prontamente mesmo com peers conectados, sem tentar
    /// notificá-los individualmente.
    #[tokio::test]
    async fn shutdown_terminates_promptly_with_connected_peers() {
        let (transport, transport_handle) = mock_transport();
        let transport: Arc<dyn P2pTransport> = Arc::new(transport);

        let (mut network_manager, command_sender, network_state) =
            NetworkManager::new(Arc::clone(&transport), open_validator(), no_op_emitter());

        network_manager.register_inbound(b"acerola/handshake/1", Arc::new(SlowHandler));

        let peer_alpha = make_peer("peer-alpha");
        let (client_alpha, server_alpha) = tokio::io::duplex(1024);
        transport_handle.inject(
            b"acerola/handshake/1",
            peer_alpha.clone(),
            client_alpha,
            server_alpha,
        );

        let manager_task_handle = tokio::spawn(network_manager.run());
        sleep(Duration::from_millis(30)).await;

        assert!(network_state.read().await.is_connected(&peer_alpha));

        let send_result = command_sender.send(NetworkCommand::Shutdown).await;
        assert!(send_result.is_ok());

        let timeout_result =
            tokio::time::timeout(Duration::from_millis(200), manager_task_handle).await;
        assert!(timeout_result.is_ok(), "shutdown should not hang with peers connected");
    }

    #[tokio::test]
    async fn switch_guard_command_updates_active_validator_and_network_mode() {
        let (transport, _transport_handle) = mock_transport();
        let (network_manager, command_sender, network_state) =
            NetworkManager::new(Arc::new(transport), open_validator(), no_op_emitter());

        let manager_task_handle = tokio::spawn(network_manager.run());

        let deny_validator: BoxedValidator = Box::new(|_ctx| {
            Box::pin(async { Err(ConnectionError::AuthDenied("switch test deny".into())) })
        });

        command_sender
            .send(NetworkCommand::SwitchGuard {
                validator: deny_validator,
                mode: crate::core::network::state::NetworkMode::Relay,
            })
            .await
            .unwrap();

        sleep(Duration::from_millis(30)).await;

        let state_guard = network_state.read().await;
        assert_eq!(*state_guard.mode(), crate::core::network::state::NetworkMode::Relay);

        let _shutdown_result = command_sender.send(NetworkCommand::Shutdown).await;
        let _manager_join_result = manager_task_handle.await;
    }

    struct TrackingBackoffTransport {
        call_timestamps: Arc<tokio::sync::Mutex<Vec<tokio::time::Instant>>>,
    }

    #[async_trait::async_trait]
    impl P2pTransport for TrackingBackoffTransport {
        fn local_id(&self) -> PeerId {
            PeerId { id: "test-peer".to_string(), device_id: None }
        }

        fn local_addr(&self) -> Result<PeerAddr, ConnectionError> {
            Ok(PeerAddr { id: self.local_id(), addrs: vec![] })
        }

        async fn accept(
            &self,
        ) -> Result<Box<dyn crate::core::transport::IncomingConnection>, ConnectionError> {
            std::future::pending().await
        }

        async fn open_bi(
            &self, _alpn: &[u8], _peer: &PeerAddr,
        ) -> Result<
            (
                Box<dyn tokio::io::AsyncWrite + Send + Unpin>,
                Box<dyn tokio::io::AsyncRead + Send + Unpin>,
            ),
            ConnectionError,
        > {
            self.call_timestamps.lock().await.push(tokio::time::Instant::now());
            Err(ConnectionError::StreamFailed("simulated network failure".to_string()))
        }

        async fn latency(&self, _peer: &PeerId) -> Option<Duration> {
            None
        }

        async fn shutdown(&self) -> Result<(), ConnectionError> {
            Ok(())
        }
    }

    #[tokio::test(start_paused = true)]
    async fn exponential_backoff_increases_delay_and_terminates_safely() {
        // Inicializa o vetor de marcas temporais para registrar o momento exato de cada tentativa
        let call_timestamps = Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let tracking_transport =
            Arc::new(TrackingBackoffTransport { call_timestamps: Arc::clone(&call_timestamps) });

        let (mut network_manager, command_sender, _network_state) =
            NetworkManager::new(tracking_transport, open_validator(), no_op_emitter());

        network_manager.register_outbound(b"acerola/test/1", Arc::new(NoopHandler));

        let manager_task_handle = tokio::spawn(network_manager.run());

        let target_peer_address = PeerAddr { id: make_peer("target-peer"), addrs: vec![] };

        command_sender
            .send(NetworkCommand::Connect {
                addr: target_peer_address,
                alpn: b"acerola/test/1".to_vec(),
            })
            .await
            .unwrap();

        // Aguarda virtualmente tempo suficiente para que todas as 5 tentativas concluam
        tokio::time::sleep(Duration::from_millis(3000)).await;

        let timestamps_guard = call_timestamps.lock().await;

        // Confirma que exatamente 5 tentativas foram executadas e que a task terminou sem pânico
        assert_eq!(timestamps_guard.len(), 5, "Exactly 5 reconnection attempts should occur");

        // Calcula os intervalos entre tentativas consecutivas
        let first_interval = timestamps_guard[1] - timestamps_guard[0];
        let second_interval = timestamps_guard[2] - timestamps_guard[1];
        let third_interval = timestamps_guard[3] - timestamps_guard[2];
        let fourth_interval = timestamps_guard[4] - timestamps_guard[3];

        // Confirma que os intervalos seguem a progressão exponencial (100ms, 200ms, 400ms, 800ms)
        assert_eq!(first_interval, Duration::from_millis(100));
        assert_eq!(second_interval, Duration::from_millis(200));
        assert_eq!(third_interval, Duration::from_millis(400));
        assert_eq!(fourth_interval, Duration::from_millis(800));

        // Verificação estrita de duplicação: se a mutação alterar * 2 para + 1, as asserções abaixo falharão
        assert_eq!(second_interval, first_interval * 2);
        assert_eq!(third_interval, second_interval * 2);
        assert_eq!(fourth_interval, third_interval * 2);

        let _shutdown_result = command_sender.send(NetworkCommand::Shutdown).await;
        let _manager_join_result = manager_task_handle.await;
    }

    struct HangingDialTransport {
        call_timestamps: Arc<tokio::sync::Mutex<Vec<tokio::time::Instant>>>,
    }

    #[async_trait::async_trait]
    impl P2pTransport for HangingDialTransport {
        fn local_id(&self) -> PeerId {
            PeerId { id: "test-peer".to_string(), device_id: None }
        }

        fn local_addr(&self) -> Result<PeerAddr, ConnectionError> {
            Ok(PeerAddr { id: self.local_id(), addrs: vec![] })
        }

        async fn accept(
            &self,
        ) -> Result<Box<dyn crate::core::transport::IncomingConnection>, ConnectionError> {
            std::future::pending().await
        }

        async fn open_bi(
            &self, _alpn: &[u8], _peer: &PeerAddr,
        ) -> Result<
            (
                Box<dyn tokio::io::AsyncWrite + Send + Unpin>,
                Box<dyn tokio::io::AsyncRead + Send + Unpin>,
            ),
            ConnectionError,
        > {
            self.call_timestamps.lock().await.push(tokio::time::Instant::now());
            // Simula um path morto que nunca responde (nem sucesso, nem erro) — o cenário real
            // de "peer trocou de rede", onde `endpoint.connect()` do iroh pode ficar preso por
            // um bom tempo tentando um endereço direto que não existe mais.
            std::future::pending().await
        }

        async fn latency(&self, _peer: &PeerId) -> Option<Duration> {
            None
        }

        async fn shutdown(&self) -> Result<(), ConnectionError> {
            Ok(())
        }
    }

    /// Regressão-alvo do bug de reconexão lenta documentado no TODO do desktop: sem um timeout
    /// próprio por tentativa, um `open_bi` que trava pra sempre (endereço cacheado morto, path
    /// nunca valida) faria a MALHA INTEIRA de retry travar na primeira tentativa — nunca
    /// chegaria a tentar as outras 4, porque o `match` externo nunca resolveria. Com
    /// `CONNECT_ATTEMPT_TIMEOUT`, cada tentativa desiste sozinha e a malha de retry consegue
    /// completar as 5 tentativas normalmente, ainda que o transporte nunca responda.
    #[tokio::test(start_paused = true)]
    async fn hanging_dial_attempt_times_out_and_all_five_retries_still_run() {
        let call_timestamps = Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let hanging_transport =
            Arc::new(HangingDialTransport { call_timestamps: Arc::clone(&call_timestamps) });

        let (mut network_manager, command_sender, _network_state) =
            NetworkManager::new(hanging_transport, open_validator(), no_op_emitter());

        network_manager.register_outbound(b"acerola/test/1", Arc::new(NoopHandler));

        let manager_task_handle = tokio::spawn(network_manager.run());

        command_sender
            .send(NetworkCommand::Connect {
                addr: PeerAddr { id: make_peer("target-peer"), addrs: vec![] },
                alpn: b"acerola/test/1".to_vec(),
            })
            .await
            .unwrap();

        // 5 tentativas de CONNECT_ATTEMPT_TIMEOUT (3s) cada, mais o backoff entre elas
        // (100+200+400+800ms) — avança bem além disso pra garantir que todas completaram.
        tokio::time::sleep(Duration::from_secs(20)).await;

        let timestamps_guard = call_timestamps.lock().await;
        assert_eq!(
            timestamps_guard.len(),
            5,
            "mesmo com open_bi travando pra sempre, o timeout por tentativa deveria permitir as 5 tentativas"
        );

        // Cada tentativa espera no máximo CONNECT_ATTEMPT_TIMEOUT antes de desistir e passar
        // pra próxima (mais o backoff, que é insignificante perto de 3s).
        for window in timestamps_guard.windows(2) {
            let elapsed = window[1] - window[0];
            assert!(
                elapsed < Duration::from_secs(4),
                "intervalo entre tentativas ({elapsed:?}) deveria ser limitado por CONNECT_ATTEMPT_TIMEOUT, não travar indefinidamente"
            );
        }

        let _ = command_sender.send(NetworkCommand::Shutdown).await;
        let _ = manager_task_handle.await;
    }

    #[tokio::test(start_paused = true)]
    async fn no_sleep_after_final_retry_attempt() {
        struct FailingTransport;
        #[async_trait::async_trait]
        impl P2pTransport for FailingTransport {
            fn local_id(&self) -> PeerId {
                make_peer("local")
            }
            fn local_addr(&self) -> Result<PeerAddr, ConnectionError> {
                Err(ConnectionError::Shutdown)
            }
            async fn accept(
                &self,
            ) -> Result<Box<dyn crate::core::transport::IncomingConnection>, ConnectionError>
            {
                std::future::pending().await
            }
            async fn open_bi(
                &self, _alpn: &[u8], _peer: &PeerAddr,
            ) -> Result<
                (
                    Box<dyn tokio::io::AsyncWrite + Send + Unpin>,
                    Box<dyn tokio::io::AsyncRead + Send + Unpin>,
                ),
                ConnectionError,
            > {
                Err(ConnectionError::StreamFailed("simulated error".to_string()))
            }
            async fn latency(&self, _peer: &PeerId) -> Option<Duration> {
                None
            }
            async fn shutdown(&self) -> Result<(), ConnectionError> {
                Ok(())
            }
        }

        let transport = Arc::new(FailingTransport);

        let (mut network_manager, command_sender, network_state) =
            NetworkManager::new(transport, open_validator(), no_op_emitter());

        network_manager.register_outbound(b"acerola/test/1", Arc::new(NoopHandler));
        let manager_task_handle = tokio::spawn(network_manager.run());

        command_sender
            .send(NetworkCommand::Connect {
                addr: PeerAddr { id: make_peer("target"), addrs: vec![] },
                alpn: b"acerola/test/1".to_vec(),
            })
            .await
            .unwrap();

        // Avança o tempo virtual em 1500ms (backoffs de retry de 100 + 200 + 400 + 800ms)
        tokio::time::sleep(Duration::from_millis(1500)).await;
        tokio::task::yield_now().await;

        // Em 1500ms, todas as 5 tentativas falharam.
        // Se attempt < max_retries foi usado, nenhum sleep ocorreu após a 5ª tentativa e a tarefa terminou,
        // reduzindo o strong_count de network_state de volta para 2 (manager + corpo do teste).
        // Se attempt <= max_retries foi usado, a tarefa estaria dormindo por 1600ms, mantendo o strong_count em 3.
        assert_eq!(
            Arc::strong_count(&network_state),
            2,
            "Outbound retry task should have completed without an extra sleep after the 5th attempt"
        );

        let _ = command_sender.send(NetworkCommand::Shutdown).await;
        let _ = manager_task_handle.await;
    }
}
