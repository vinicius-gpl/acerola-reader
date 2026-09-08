use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use secrecy::ExposeSecret;
use tokio::sync::RwLock;

use super::acerola_p2p::AcerolaP2p;
use crate::{
    core::{
        guard::BoxedValidator,
        network::{manager::NetworkManager, state::NetworkState},
        storage::P2PStorage,
        transport::{P2pTransport, TransportP2pBuilder},
    },
    data::{
        identity::device_info::DeviceInfo,
        protocol::{
            rpc::{RpcClientHandler, RpcServerHandler},
            DeviceInfoStore, EventEmitter, ProtocolHandler,
        },
    },
    infra::{error::ConnectionError, peer::PeerId},
};

/// `DeviceInfoStore` que grava tanto no `NetworkState` em memória quanto no `P2PStorage`
/// (quando configurado) — sem isso, o nome de um peer pareado só sobrevive enquanto o
/// processo estiver de pé, some ao reiniciar o app até o próximo handshake com aquele peer
/// (ver `restore_cached_device_info`, que resolve a metade "carregar de volta no boot" disso).
/// Falha ao persistir é só logada, não propagada: um nome que não gravou no disco não deveria
/// derrubar um handshake que já completou com sucesso em memória.
struct PersistentDeviceInfoStore {
    state: Arc<RwLock<NetworkState>>,
    storage: Option<Arc<dyn P2PStorage>>,
}

#[async_trait]
impl DeviceInfoStore for PersistentDeviceInfoStore {
    async fn store_device_info(&self, peer: PeerId, info: DeviceInfo) {
        self.state.write().await.store_device_info(peer.clone(), info.clone());

        if let Some(storage) = &self.storage {
            if let Err(err) = storage.save_device_info(&peer, &info).await {
                tracing::warn!(peer = %peer.id, error = ?err, "failed to persist device info");
            }
        }
    }
}

const RESERVED_ALPNS: &[&[u8]] = &[b"acerola/handshake/1"];

/// Estrutura auxiliar para pré-configurar o ecossistema P2p antes da iniciação real no sistema operacional.
///
/// Através desse builder é possível injetar regras de firewall,
/// registrar portas e protocols customizados (handlers ALPN) e repassar
/// as lógicas de monitoria pro usuário.
pub struct AcerolaP2pBuilder<TB: TransportP2pBuilder>
where
    TB::Output: 'static,
{
    pub(super) transport: TB,
    pub(super) emit: EventEmitter,
    pub(super) device_info: DeviceInfo,
    pub(super) guard: BoxedValidator,
    pub(super) storage: Option<Arc<dyn P2PStorage>>,
    pub(super) handlers_inbound: HashMap<Vec<u8>, Arc<dyn ProtocolHandler>>,
    pub(super) handlers_outbound: HashMap<Vec<u8>, Arc<dyn ProtocolHandler>>,
}

impl<TB: TransportP2pBuilder> AcerolaP2pBuilder<TB> {
    pub(super) fn new(emit: EventEmitter, transport: TB, device_info: DeviceInfo) -> Self {
        Self {
            emit,
            transport,
            device_info,
            handlers_inbound: HashMap::new(),
            handlers_outbound: HashMap::new(),
            guard: Box::new(|_ctx| Box::pin(async { Ok(()) })),
            storage: None,
        }
    }

    /// Atribui um componente ou closure Guard para checagem estrita de cada handshake na rede.
    pub fn guard(mut self, validator: BoxedValidator) -> Self {
        self.guard = validator;
        self
    }

    /// Injeta uma implementação de `P2PStorage` para persistência de chaves de identidade e cache de peers.
    pub fn storage(mut self, storage: impl P2PStorage + 'static) -> Self {
        self.storage = Some(Arc::new(storage));
        self
    }

    /// Acopla um manipulador passivo de requisições de serviço à pilha.
    /// Dispara somente quando um par iniciar conexão invocando a exata chave `alpn`.
    pub fn inbound(mut self, alpn: &[u8], handler: Arc<dyn ProtocolHandler>) -> Self {
        assert!(
            !RESERVED_ALPNS.contains(&alpn),
            "ALPN {:?} is reserved by the library and cannot be overridden",
            alpn
        );
        self.handlers_inbound.insert(alpn.to_vec(), handler);
        self
    }

    /// Acopla um manipulador proativo à pilha, a ser usado toda vez que o software
    /// quiser ativamente invocar um sub-serviço e processar a via dupla ativamente.
    pub fn outbound(mut self, alpn: &[u8], handler: Arc<dyn ProtocolHandler>) -> Self {
        assert!(
            !RESERVED_ALPNS.contains(&alpn),
            "ALPN {:?} is reserved by the library and cannot be overridden",
            alpn
        );
        self.handlers_outbound.insert(alpn.to_vec(), handler);
        self
    }

    /// Sincroniza a chave/seed de identidade entre o transport e o storage configurado.
    ///
    /// Distingue "nunca foi salvo" (`Ok(None)` — primeira execução legítima, gera uma
    /// identidade nova normalmente) de "falha real ao carregar" (`Err` — ex: a chave mestra
    /// mudou e o blob não descriptografa mais). O segundo caso NUNCA pode cair no mesmo
    /// caminho do primeiro: gerar e salvar uma identidade nova ali sobrescreveria
    /// silenciosamente `identity.enc` na primeira falha transitória de storage/keyring,
    /// destruindo a identidade (e todo pareamento associado a ela) sem nenhum aviso.
    async fn resolve_identity(&mut self) -> Result<(), ConnectionError> {
        let Some(storage) = &self.storage else { return Ok(()) };

        match storage.load_identity().await {
            Ok(Some(bytes)) => {
                if let Ok(seed) = bytes.try_into() {
                    self.transport.set_seed(seed);
                    return Ok(());
                }
                // Tamanho inesperado (não 32 bytes): dado corrompido num nível diferente de
                // uma falha de descriptografia — mantém o comportamento já existente de
                // regenerar, coberto por `resolve_identity_regenerates_seed_when_storage_returns_invalid_bytes`.
            },
            Ok(None) => {
                // Storage respondeu, e não há identidade salva ainda — primeira execução.
            },
            Err(err) => return Err(err),
        }

        let seed = match self.transport.get_seed() {
            Some(seed) => seed,
            None => *crate::data::identity::generate_seed()?.expose_secret(),
        };

        self.transport.set_seed(seed);
        storage.save_identity(&seed).await
    }

    /// Carrega e registra peers salvos no cache de storage no estado inicial da rede.
    async fn restore_cached_peers(
        storage: Option<&Arc<dyn P2PStorage>>,
        state: &tokio::sync::RwLock<crate::core::network::state::NetworkState>,
    ) {
        if let Some(storage) = storage {
            if let Ok(cached_peers) = storage.load_peers().await {
                state.write().await.store_peer_addrs(cached_peers);
            }
        }
    }

    /// Carrega e registra os nomes de dispositivo salvos no cache de storage no estado
    /// inicial da rede — contraparte de `restore_cached_peers` para `DeviceInfo`. Sem isso,
    /// mesmo persistindo em disco (`PersistentDeviceInfoStore`), o nome de um peer pareado
    /// ficaria em branco até aquele peer específico reconectar nesta sessão.
    async fn restore_cached_device_info(
        storage: Option<&Arc<dyn P2PStorage>>,
        state: &tokio::sync::RwLock<crate::core::network::state::NetworkState>,
    ) {
        if let Some(storage) = storage {
            if let Ok(cached_device_info) = storage.load_device_info().await {
                let mut state = state.write().await;
                for (peer, info) in cached_device_info {
                    state.store_device_info(peer, info);
                }
            }
        }
    }

    /// Compila as configurações submetidas e consolida a interface física no sistema operacional (abre as sockets).
    ///
    /// Além de popular a estrutura do `NetworkManager`, ativa de ofício o handler base `acerola/handshake/1`.
    pub async fn build(mut self) -> Result<AcerolaP2p, ConnectionError> {
        self.resolve_identity().await?;

        let alpns: Vec<Vec<u8>> = RESERVED_ALPNS
            .iter()
            .map(|it| it.to_vec())
            .chain(self.handlers_inbound.keys().cloned())
            .chain(self.handlers_outbound.keys().cloned())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let transport = Arc::new(self.transport.build(alpns).await?);

        let local_id = transport.local_id();
        // Só valida aqui que o transporte já consegue montar um `EndpointAddr` válido — o
        // valor em si não é guardado (`AcerolaP2p::local_addr` recalcula ao vivo a cada
        // chamada, ver doc lá).
        transport.local_addr()?;

        let (mut manager, command_tx, state) = NetworkManager::with_storage(
            Arc::clone(&transport) as Arc<dyn P2pTransport>,
            self.guard,
            Arc::clone(&self.emit),
            self.storage.clone(),
        );

        Self::restore_cached_peers(self.storage.as_ref(), &state).await;
        Self::restore_cached_device_info(self.storage.as_ref(), &state).await;

        // Compartilhado com `AcerolaP2p` (não clonado por valor) — é essa referência comum que
        // permite `set_local_device_name` valer no próximo handshake sem reconstruir os
        // handlers nem reiniciar o node.
        let device_info = Arc::new(RwLock::new(self.device_info));

        let device_info_store: Arc<dyn DeviceInfoStore> = Arc::new(PersistentDeviceInfoStore {
            state: Arc::clone(&state),
            storage: self.storage.clone(),
        });

        manager.register_inbound(
            b"acerola/handshake/1",
            Arc::new(RpcServerHandler::new(
                Arc::clone(&self.emit),
                Arc::clone(&device_info),
                Arc::clone(&device_info_store),
            )),
        );

        manager.register_outbound(
            b"acerola/handshake/1",
            Arc::new(RpcClientHandler::new(
                Arc::clone(&self.emit),
                Arc::clone(&device_info),
                Arc::clone(&device_info_store),
            )),
        );

        self.handlers_inbound.into_iter().for_each(|(alpn, handler)| {
            manager.register_inbound(&alpn, handler);
        });

        self.handlers_outbound.into_iter().for_each(|(alpn, handler)| {
            manager.register_outbound(&alpn, handler);
        });

        let run_handle = tokio::spawn(manager.run());
        Ok(AcerolaP2p {
            command_tx,
            local_id,
            state,
            device_info,
            transport: transport as Arc<dyn P2pTransport>,
            run_handle: tokio::sync::Mutex::new(Some(run_handle)),
        })
    }
}

#[cfg(all(test, feature = "iroh"))]
mod tests {
    use tokio::io::{AsyncRead, AsyncWrite};

    use super::*;
    use crate::{
        core::{storage::InMemoryStorage, transport::iroh::IrohTransportBuilder},
        data::identity::device_info::DeviceInfo,
        infra::peer::PeerId,
    };

    fn no_op_emitter() -> EventEmitter {
        Arc::new(|_event: &str, _payload: String| {})
    }

    fn test_device_info() -> DeviceInfo {
        DeviceInfo {
            name: "test-device".to_string(),
            os: "linux".to_string(),
            version: "0.0.1".to_string(),
        }
    }

    struct NoOpHandler;

    #[async_trait::async_trait]
    impl ProtocolHandler for NoOpHandler {
        async fn handle(
            &self, _peer: &PeerId, _send: Box<dyn AsyncWrite + Send + Unpin>,
            _recv: Box<dyn AsyncRead + Send + Unpin>,
        ) -> Result<(), ConnectionError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn build_returns_valid_node() {
        assert!(AcerolaP2p::builder(
            no_op_emitter(),
            IrohTransportBuilder::default(),
            test_device_info()
        )
        .build()
        .await
        .is_ok());
    }

    #[test]
    #[should_panic(expected = "reserved by the library")]
    fn inbound_with_reserved_alpn_causes_panic() {
        AcerolaP2p::builder(no_op_emitter(), IrohTransportBuilder::default(), test_device_info())
            .inbound(b"acerola/handshake/1", Arc::new(NoOpHandler));
    }

    #[test]
    #[should_panic(expected = "reserved by the library")]
    fn outbound_with_reserved_alpn_causes_panic() {
        AcerolaP2p::builder(no_op_emitter(), IrohTransportBuilder::default(), test_device_info())
            .outbound(b"acerola/handshake/1", Arc::new(NoOpHandler));
    }

    #[tokio::test]
    async fn build_with_custom_handler_does_not_fail() {
        let result = AcerolaP2p::builder(
            no_op_emitter(),
            IrohTransportBuilder::default(),
            test_device_info(),
        )
        .inbound(b"my/protocol", Arc::new(NoOpHandler))
        .outbound(b"my/protocol", Arc::new(NoOpHandler))
        .build()
        .await;

        assert!(result.is_ok());
    }

    fn capture_emitter() -> (EventEmitter, Arc<std::sync::Mutex<Vec<String>>>) {
        let events = Arc::new(std::sync::Mutex::new(Vec::new()));
        let clone = Arc::clone(&events);
        let emit: EventEmitter = Arc::new(move |event: &str, _: String| {
            clone.lock().unwrap().push(event.to_string());
        });
        (emit, events)
    }

    #[tokio::test]
    async fn reserved_handshake_completes_between_two_nodes() {
        let (emit_a, events_a) = capture_emitter();
        let (emit_b, events_b) = capture_emitter();

        let node_a =
            AcerolaP2p::builder(emit_a, IrohTransportBuilder::default(), test_device_info())
                .build()
                .await
                .unwrap();

        let node_b =
            AcerolaP2p::builder(emit_b, IrohTransportBuilder::default(), test_device_info())
                .build()
                .await
                .unwrap();

        let addr_b = node_b.local_addr().unwrap();

        // Aguarda mDNS descobrir o peer antes de tentar conectar
        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
        node_a.connect(addr_b, b"acerola/handshake/1").await.unwrap();

        // Aguarda handshake completar
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        let ev_a = events_a.lock().unwrap();
        let ev_b = events_b.lock().unwrap();

        assert!(ev_a.iter().any(|e| e == "rpc:ping_sent"), "node A: ping sent");
        assert!(ev_a.iter().any(|e| e == "rpc:pong_received"), "node A: pong received");
        assert!(
            ev_a.iter().any(|e| e == "rpc:device_info_received"),
            "node A: device info received"
        );

        assert!(ev_b.iter().any(|e| e == "rpc:ping_received"), "node B: ping received");
        assert!(ev_b.iter().any(|e| e == "rpc:pong_sent"), "node B: pong sent");
        assert!(
            ev_b.iter().any(|e| e == "rpc:device_info_exchanged"),
            "node B: device info exchanged"
        );
    }

    #[tokio::test]
    async fn storage_persists_identity_across_rebuild() {
        let storage = InMemoryStorage::new();

        let node1 = AcerolaP2p::builder(
            no_op_emitter(),
            IrohTransportBuilder::default(),
            test_device_info(),
        )
        .storage(storage.clone())
        .build()
        .await
        .unwrap();

        let id1 = node1.local_id().to_string();
        let dev_id1 = node1.local_device_id().map(|s| s.to_string());

        let node2 = AcerolaP2p::builder(
            no_op_emitter(),
            IrohTransportBuilder::default(),
            test_device_info(),
        )
        .storage(storage)
        .build()
        .await
        .unwrap();

        let id2 = node2.local_id().to_string();
        let dev_id2 = node2.local_device_id().map(|s| s.to_string());

        assert_eq!(id1, id2);
        assert_eq!(dev_id1, dev_id2);
    }

    struct FailingPeerLoadStorage;

    #[async_trait::async_trait]
    impl crate::core::storage::P2PStorage for FailingPeerLoadStorage {
        async fn save_identity(&self, _secret: &[u8]) -> Result<(), ConnectionError> {
            Ok(())
        }
        async fn load_identity(&self) -> Result<Option<Vec<u8>>, ConnectionError> {
            Ok(None)
        }
        async fn save_peer(
            &self, _peer: &crate::infra::peer::PeerAddr,
        ) -> Result<(), ConnectionError> {
            Ok(())
        }
        async fn load_peers(&self) -> Result<Vec<crate::infra::peer::PeerAddr>, ConnectionError> {
            Err(ConnectionError::StreamFailed("disk read error".to_string()))
        }
        async fn save_device_info(
            &self, _peer: &PeerId, _info: &DeviceInfo,
        ) -> Result<(), ConnectionError> {
            Ok(())
        }
        async fn load_device_info(&self) -> Result<Vec<(PeerId, DeviceInfo)>, ConnectionError> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn failing_peer_load_storage_returns_expected_results() {
        let storage = FailingPeerLoadStorage;
        assert_eq!(storage.load_identity().await.unwrap(), None);
        assert!(matches!(storage.load_peers().await, Err(ConnectionError::StreamFailed(_))));
    }

    #[tokio::test]
    async fn restore_cached_peers_handles_storage_failure_gracefully() {
        // Verifica que o erro retornado por load_peers não causa panic e deixa o estado limpo
        let failing_storage: Arc<dyn crate::core::storage::P2PStorage> =
            Arc::new(FailingPeerLoadStorage);
        let network_state =
            Arc::new(tokio::sync::RwLock::new(crate::core::network::state::NetworkState::new()));

        AcerolaP2pBuilder::<IrohTransportBuilder>::restore_cached_peers(
            Some(&failing_storage),
            &network_state,
        )
        .await;

        let state_guard = network_state.read().await;
        assert!(state_guard.peers().is_empty());
    }

    #[tokio::test]
    async fn restore_cached_peers_populates_network_state_with_cached_peers() {
        // Inicializa storage contendo um peer preexistente no cache
        let memory_storage = InMemoryStorage::new();
        let cached_peer_address = crate::infra::peer::PeerAddr {
            id: PeerId { id: "cached-peer-1".to_string(), device_id: None },
            addrs: vec![],
        };
        memory_storage.save_peer(&cached_peer_address).await.unwrap();

        let network_state =
            Arc::new(tokio::sync::RwLock::new(crate::core::network::state::NetworkState::new()));

        let storage_arc: Arc<dyn crate::core::storage::P2PStorage> = Arc::new(memory_storage);

        // Executa a restauração de peers salvos
        AcerolaP2pBuilder::<IrohTransportBuilder>::restore_cached_peers(
            Some(&storage_arc),
            &network_state,
        )
        .await;

        // Verifica que o peer foi carregado para a tabela de endereços conhecidos do estado
        let state_guard = network_state.read().await;
        assert!(state_guard.get_addr(&cached_peer_address.id).is_some());
    }

    #[tokio::test]
    async fn restore_cached_device_info_populates_network_state_with_cached_names() {
        let memory_storage = InMemoryStorage::new();
        let cached_peer = PeerId { id: "cached-peer-1".to_string(), device_id: None };
        memory_storage
            .save_device_info(
                &cached_peer,
                &DeviceInfo {
                    name: "PC do Vinicius".to_string(),
                    os: "windows".to_string(),
                    version: "1.0.0".to_string(),
                },
            )
            .await
            .unwrap();

        let network_state =
            Arc::new(tokio::sync::RwLock::new(crate::core::network::state::NetworkState::new()));
        let storage_arc: Arc<dyn crate::core::storage::P2PStorage> = Arc::new(memory_storage);

        AcerolaP2pBuilder::<IrohTransportBuilder>::restore_cached_device_info(
            Some(&storage_arc),
            &network_state,
        )
        .await;

        let state_guard = network_state.read().await;
        assert_eq!(state_guard.get_device_info(&cached_peer).unwrap().name, "PC do Vinicius");
    }

    #[tokio::test]
    async fn persistent_device_info_store_persists_to_storage_and_memory() {
        let storage = InMemoryStorage::new();
        let storage_arc: Arc<dyn crate::core::storage::P2PStorage> = Arc::new(storage);
        let network_state =
            Arc::new(tokio::sync::RwLock::new(crate::core::network::state::NetworkState::new()));

        let store = PersistentDeviceInfoStore {
            state: Arc::clone(&network_state),
            storage: Some(Arc::clone(&storage_arc)),
        };

        let peer = PeerId { id: "peer-1".to_string(), device_id: None };
        let info = DeviceInfo {
            name: "Notebook".to_string(),
            os: "linux".to_string(),
            version: "2.0.0".to_string(),
        };
        store.store_device_info(peer.clone(), info.clone()).await;

        let state_guard = network_state.read().await;
        assert_eq!(state_guard.get_device_info(&peer).unwrap().name, "Notebook");
        drop(state_guard);

        let persisted = storage_arc.load_device_info().await.unwrap();
        assert_eq!(persisted.len(), 1);
        assert_eq!(persisted[0].0, peer);
        assert_eq!(persisted[0].1.name, "Notebook");
    }

    struct InvalidSeedStorage {
        invalid_bytes: Vec<u8>,
        saved_seed: Arc<std::sync::Mutex<Option<Vec<u8>>>>,
    }

    #[async_trait::async_trait]
    impl crate::core::storage::P2PStorage for InvalidSeedStorage {
        async fn save_identity(&self, secret: &[u8]) -> Result<(), ConnectionError> {
            let mut saved_seed_guard = self.saved_seed.lock().unwrap();
            *saved_seed_guard = Some(secret.to_vec());
            Ok(())
        }

        async fn load_identity(&self) -> Result<Option<Vec<u8>>, ConnectionError> {
            Ok(Some(self.invalid_bytes.clone()))
        }

        async fn save_peer(
            &self, _peer: &crate::infra::peer::PeerAddr,
        ) -> Result<(), ConnectionError> {
            Ok(())
        }

        async fn load_peers(&self) -> Result<Vec<crate::infra::peer::PeerAddr>, ConnectionError> {
            Ok(vec![])
        }
        async fn save_device_info(
            &self, _peer: &PeerId, _info: &DeviceInfo,
        ) -> Result<(), ConnectionError> {
            Ok(())
        }
        async fn load_device_info(&self) -> Result<Vec<(PeerId, DeviceInfo)>, ConnectionError> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn invalid_seed_storage_returns_configured_bytes() {
        let container = Arc::new(std::sync::Mutex::new(None));
        let storage =
            InvalidSeedStorage { invalid_bytes: vec![1, 2, 3, 4, 5], saved_seed: container };
        assert_eq!(storage.load_identity().await.unwrap().unwrap(), vec![1, 2, 3, 4, 5]);
    }

    #[tokio::test]
    async fn resolve_identity_regenerates_seed_when_storage_returns_invalid_bytes() {
        // Inicializa storage mock que retorna bytes de tamanho inválido (5 bytes em vez de 32)
        let saved_seed_container = Arc::new(std::sync::Mutex::new(None));
        let invalid_seed_storage = InvalidSeedStorage {
            invalid_bytes: vec![1, 2, 3, 4, 5],
            saved_seed: Arc::clone(&saved_seed_container),
        };

        let mut builder = AcerolaP2pBuilder::new(
            no_op_emitter(),
            IrohTransportBuilder::default(),
            test_device_info(),
        )
        .storage(invalid_seed_storage);

        // Executa a resolução de identidade em isolamento
        let resolution_result = builder.resolve_identity().await;
        assert!(resolution_result.is_ok());

        // Verifica que a seed gerada e atribuída ao transporte possui exatamente 32 bytes
        let transport_seed = builder.transport.get_seed();
        assert!(transport_seed.is_some());
        let resolved_seed_bytes = transport_seed.unwrap();
        assert_eq!(resolved_seed_bytes.len(), 32);

        // Verifica que a nova seed válida de 32 bytes foi persistida no storage para corrigir a seed inválida anterior
        let saved_seed_guard = saved_seed_container.lock().unwrap();
        assert!(saved_seed_guard.is_some());
        let persisted_bytes = saved_seed_guard.as_ref().unwrap();
        assert_eq!(persisted_bytes.len(), 32);
        assert_ne!(persisted_bytes.as_slice(), &[1, 2, 3, 4, 5]);
        assert_eq!(persisted_bytes.as_slice(), resolved_seed_bytes.as_slice());
    }

    struct FailingIdentityLoadStorage {
        save_identity_called: Arc<std::sync::atomic::AtomicBool>,
    }

    #[async_trait::async_trait]
    impl crate::core::storage::P2PStorage for FailingIdentityLoadStorage {
        async fn save_identity(&self, _secret: &[u8]) -> Result<(), ConnectionError> {
            self.save_identity_called.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
        async fn load_identity(&self) -> Result<Option<Vec<u8>>, ConnectionError> {
            Err(ConnectionError::StreamFailed("decrypt failed: wrong master key".to_string()))
        }
        async fn save_peer(
            &self, _peer: &crate::infra::peer::PeerAddr,
        ) -> Result<(), ConnectionError> {
            Ok(())
        }
        async fn load_peers(&self) -> Result<Vec<crate::infra::peer::PeerAddr>, ConnectionError> {
            Ok(vec![])
        }
        async fn save_device_info(
            &self, _peer: &PeerId, _info: &DeviceInfo,
        ) -> Result<(), ConnectionError> {
            Ok(())
        }
        async fn load_device_info(&self) -> Result<Vec<(PeerId, DeviceInfo)>, ConnectionError> {
            Ok(vec![])
        }
    }

    /// Regressão: uma falha real ao carregar a identidade (ex: chave mestra trocou e o blob
    /// não descriptografa mais) não pode terminar em "gera identidade nova e sobrescreve" —
    /// esse foi exatamente o bug que apagou o pareamento de um usuário em produção. O erro
    /// tem que propagar, e `save_identity` nunca pode ser chamado nesse caminho.
    #[tokio::test]
    async fn resolve_identity_propagates_error_instead_of_silently_regenerating() {
        let save_identity_called = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let failing_storage =
            FailingIdentityLoadStorage { save_identity_called: Arc::clone(&save_identity_called) };

        let mut builder = AcerolaP2pBuilder::new(
            no_op_emitter(),
            IrohTransportBuilder::default(),
            test_device_info(),
        )
        .storage(failing_storage);

        let resolution_result = builder.resolve_identity().await;

        assert!(resolution_result.is_err(), "loading failure must propagate, not be swallowed");
        assert!(builder.transport.get_seed().is_none(), "no new seed should have been assigned");
        assert!(
            !save_identity_called.load(std::sync::atomic::Ordering::SeqCst),
            "save_identity should never be called when load_identity truly failed"
        );
    }
}
