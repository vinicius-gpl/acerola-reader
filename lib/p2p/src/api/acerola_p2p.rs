use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use tokio::sync::{mpsc, RwLock};

use super::acerola_builder::AcerolaP2pBuilder;
use crate::{
    core::{
        blobs::P2pBlobStore,
        guard::BoxedValidator,
        network::{
            manager::NetworkCommand,
            state::{NetworkMode, NetworkState},
        },
        transport::{P2pTransport, TransportP2pBuilder},
    },
    data::{identity::device_info::DeviceInfo, protocol::EventEmitter},
    infra::{
        error::ConnectionError,
        peer::{PeerAddr, PeerId},
    },
};

/// A Instância consolidada e o Controlador do Nó rodando em background.
///
/// É com este objeto instanciado que a aplicação cliente irá de fato provocar a rede,
/// discando ativamente para outros nós, requisitando o estado global ou mandando
/// comandos administrativos de desligamento ou reorientação (SwitchGuard).
pub struct AcerolaP2p {
    pub(super) command_tx: mpsc::Sender<NetworkCommand>,
    pub(super) state: Arc<RwLock<NetworkState>>,
    pub(super) device_info: Arc<RwLock<DeviceInfo>>,
    pub(super) local_id: PeerId,
    pub(super) transport: Arc<dyn P2pTransport>,
    /// `JoinHandle` da task de `NetworkManager::run()` (`tokio::spawn` em `acerola_builder.rs`,
    /// descartada até esta mudança) — guardada só pra `shutdown()` poder confirmar que o loop
    /// terminou de verdade antes de devolver, em vez de mandar `NetworkCommand::Shutdown` e
    /// torcer. `Mutex` (não `RwLock`) porque só é tomada (`.take()`) uma vez, nunca lida
    /// concorrentemente; `tokio::sync` porque `.lock()` precisa atravessar o `.await` do join.
    pub(super) run_handle: tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl AcerolaP2p {
    /// Único ponto de partida da API, devolve uma estrutura passível de configuração.
    pub fn builder<TB: TransportP2pBuilder>(
        emit: EventEmitter, transport: TB, device_info: DeviceInfo,
    ) -> AcerolaP2pBuilder<TB>
    where
        TB::Output: 'static,
    {
        AcerolaP2pBuilder::new(emit, transport, device_info)
    }

    /// Retorna a string legível (Base32/Base64, dependendo do backend Iroh) do nó residente.
    pub fn local_id(&self) -> &str {
        &self.local_id.id
    }

    /// Retorna um objeto do PeerAddr, juntamente vem o id do peerid + addr — recalculado a
    /// partir do transporte vivo a cada chamada, nunca cacheado. Antes disso, o valor era
    /// capturado uma única vez na construção do node e guardado num campo estático: qualquer
    /// endereço direto/relay que mudasse depois (troca de rede, `apply_relay_mode`) nunca se
    /// refletia aqui, então um QR code de pareamento gerado depois de uma troca de relay ao
    /// vivo continuava anunciando o relay ANTIGO — o outro lado disca certinho pro endereço
    /// que recebeu, só que ele já não é mais onde o node está de fato escutando.
    pub fn local_addr(&self) -> Result<PeerAddr, ConnectionError> {
        self.transport.local_addr()
    }

    /// Retorna o `device_id` UUID v5 derivado da chave pública do nó.
    pub fn local_device_id(&self) -> Option<&str> {
        self.local_id.device_id.as_deref()
    }

    /// Retorna os metadados do dispositivo local informados no builder (ou o apelido
    /// definido depois via [`Self::set_local_device_name`]).
    pub async fn local_device_info(&self) -> DeviceInfo {
        self.device_info.read().await.clone()
    }

    /// Sobrescreve o nome exibido do dispositivo local (ex: apelido custom estilo LocalSend em
    /// vez do hostname). Vale a partir do próximo handshake — `RpcServerHandler`/
    /// `RpcClientHandler` compartilham a mesma referência e leem o valor atual a cada troca de
    /// `DeviceInfo`, então não é preciso reiniciar o node pra um peer novo já ver o nome novo.
    pub async fn set_local_device_name(&self, name: String) {
        self.device_info.write().await.name = name;
    }

    /// Pede ativamente ao daemon de gerência para abrir um pipe QUIC até um determinado Nó.
    ///
    /// Se a resposta for exitosa, as transmissões vão direto pro handler do protocolo (`alpn`) mapeado.
    #[rustfmt::skip]
    pub async fn connect(&self, addr: PeerAddr, alpn: &[u8]) -> Result<(), ConnectionError> {
        self.command_tx.send(NetworkCommand::Connect { addr, alpn: alpn.to_vec() }).await.map_err(|_| ConnectionError::Shutdown)
    }

    /// Captura um Snapshot/Cópia pesada dos nós que estão trafegando e seus marcadores de sub-protocolo atrelados.
    pub async fn connected_peers(&self) -> HashMap<PeerId, HashSet<Vec<u8>>> {
        self.state.read().await.peers().clone()
    }

    /// Captura peers conectados junto com suas informações de dispositivo, em lock único.
    pub async fn connected_peers_with_info(
        &self,
    ) -> Vec<(PeerId, HashSet<Vec<u8>>, Option<DeviceInfo>)> {
        let state = self.state.read().await;
        state
            .peers()
            .iter()
            .map(|(peer, alpns)| {
                (peer.clone(), alpns.clone(), state.get_device_info(peer).cloned())
            })
            .collect()
    }

    /// Extrai o modo da interface (Local/Relay) operando no momento.
    pub async fn mode(&self) -> NetworkMode {
        self.state.read().await.mode().clone()
    }

    /// Retorna `true` se há uma conexão física viva com este peer neste exato instante.
    ///
    /// Diferente de `connected_peers()` (que reflete sessões de protocolo pontuais, como um
    /// handshake que já terminou), isto consulta o transporte diretamente — o QUIC do iroh
    /// mantém a conexão viva sozinho via keepalive nativo, então este sinal não desaparece só
    /// porque nenhum handler está rodando. É o bloco de construção certo para uma visão
    /// "dispositivos online agora" (ex: lista de peers conectados na UI).
    pub async fn is_peer_reachable(&self, peer: &PeerId) -> bool {
        self.transport.is_connected(peer).await
    }

    /// RTT medido da conexão física viva com `peer`, ou `None` sem uma conexão pra medir —
    /// mesmo valor que alimenta o evento periódico `network:latency` (`core/network/manager.rs`),
    /// exposto sob demanda pra quem precisa de uma leitura fresca no início de uma transferência
    /// (ex.: dimensionar quantos fetches de blob rodar em paralelo) sem esperar o próximo tick
    /// do intervalo de 30s.
    pub async fn latency(&self, peer: &PeerId) -> Option<std::time::Duration> {
        self.transport.latency(peer).await
    }

    /// Notifica o transporte de uma possível mudança de rede do SO — ver a doc em
    /// `P2pTransport::network_change` para o motivo de isso ser necessário no Android.
    pub async fn network_change(&self) {
        self.transport.network_change().await;
    }

    /// Aplica um novo modo de relay ao node já vivo — ver a doc em
    /// `P2pTransport::apply_relay_mode` para o porquê disso não precisar reconstruir o node
    /// inteiro. Chamado sempre que o usuário muda a config de relay na UI, em vez de exigir
    /// fechar e reabrir o app pra valer.
    #[cfg(feature = "iroh")]
    pub async fn apply_relay_mode(
        &self, config: crate::core::transport::iroh::RelayModeConfig,
    ) -> Result<(), ConnectionError> {
        self.transport.apply_relay_mode(config).await
    }

    /// Retorna o store de blobs deste nó, se o adapter de transporte suportar essa capacidade
    /// (ex: `IrohTransport` construído com `.blobs(...)`). `None` quando nenhum adapter de blob
    /// foi configurado.
    pub async fn blobs(&self) -> Option<Arc<dyn P2pBlobStore>> {
        self.transport.blobs().await
    }

    /// Todos os peers já vistos nesta sessão (endereço + device info de um handshake anterior),
    /// independente de estarem alcançáveis neste instante — use `is_peer_reachable` para saber
    /// se um peer específico desta lista está online agora.
    pub async fn known_peers(&self) -> Vec<(PeerId, PeerAddr, Option<DeviceInfo>)> {
        let state = self.state.read().await;
        state
            .known_peers()
            .map(|(peer, addr, info)| (peer.clone(), addr.clone(), info.cloned()))
            .collect()
    }

    /// Solicita em runtime ao Daemon a permuta de middleware de restrição sem cair nenhuma thread.
    ///
    /// Ao receber o novo validator o gerente recusa/aceita instantes novas conexões sem derrubar os sockets mantidos.
    #[rustfmt::skip]
    pub async fn switch_guard(
        &self, validator: BoxedValidator, mode: NetworkMode,
    ) -> Result<(), ConnectionError> {
        self.command_tx.send(NetworkCommand::SwitchGuard { validator, mode }).await.map_err(|_| ConnectionError::Shutdown)
    }

    /// Aciona a sequência final de drenagem forçando o cancelamento do background Event Loop
    /// e do serviço nativo na memória.
    ///
    /// Diferente de antes, só retorna depois que a task de `NetworkManager::run()` de fato
    /// terminou (incluindo `transport.shutdown()`, que fecha o `Endpoint` real do iroh) — não
    /// só depois que o comando foi enfileirado. Isso importa pra quem quer "reiniciar" o node
    /// (shutdown + `builder().build()` de novo): sem essa garantia, o socket UDP antigo podia
    /// ainda estar de pé quando o novo node tentasse bindar, ou continuar recebendo tráfego
    /// que deveria ir pro node novo.
    ///
    /// Ignora erro de `send` (canal fechado significa que o manager já parou sozinho — não é
    /// motivo pra desistir de confirmar o teardown) e é seguro chamar mais de uma vez: a
    /// segunda chamada só encontra `run_handle` já vazio (`.take()` na primeira) e retorna
    /// `Ok(())` imediatamente.
    pub async fn shutdown(&self) -> Result<(), ConnectionError> {
        let _ = self.command_tx.send(NetworkCommand::Shutdown).await;

        if let Some(handle) = self.run_handle.lock().await.take() {
            // Diferente do `send` acima, isto NÃO é um erro esperado em uso normal: só falha se
            // `NetworkManager::run()` entrou em pânico ou foi cancelada. Descartar em silêncio
            // deixaria exatamente o tipo de falha que essa função existe pra detectar (task
            // morreu sem de fato rodar `transport.shutdown()`) invisível pro chamador.
            if let Err(error) = handle.await {
                tracing::warn!(?error, "network manager task did not shut down cleanly");
            }
        }

        Ok(())
    }
}

#[cfg(all(test, feature = "iroh"))]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::{
        core::transport::iroh::IrohTransportBuilder, data::identity::device_info::DeviceInfo,
    };

    fn no_op_emitter() -> EventEmitter {
        Arc::new(|_event: &str, _payload: String| {})
    }

    fn capture_emitter() -> (EventEmitter, Arc<Mutex<Vec<String>>>) {
        let events = Arc::new(Mutex::new(Vec::new()));
        let clone = Arc::clone(&events);
        let emit: EventEmitter = Arc::new(move |event: &str, _: String| {
            clone.lock().unwrap().push(event.to_string());
        });
        (emit, events)
    }

    fn test_device_info() -> DeviceInfo {
        DeviceInfo {
            name: "test-device".to_string(),
            os: "linux".to_string(),
            version: "0.0.1".to_string(),
        }
    }

    async fn build_node() -> AcerolaP2p {
        AcerolaP2p::builder(no_op_emitter(), IrohTransportBuilder::default(), test_device_info())
            .build()
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn local_id_not_empty() {
        let node = build_node().await;
        assert!(!node.local_id().is_empty());
    }

    #[tokio::test]
    async fn connected_peers_start_empty() {
        let node = build_node().await;
        assert!(node.connected_peers().await.is_empty());
    }

    #[tokio::test]
    async fn shutdown_without_error() {
        let node = build_node().await;
        assert!(node.shutdown().await.is_ok());
    }

    /// `shutdown()` agora espera `run_handle` de verdade (`.take()`, ver doc do método) —
    /// chamar de novo depois não pode travar nem falhar: a segunda chamada só encontra
    /// `run_handle` já vazio e retorna `Ok(())` na hora. Regressão específica pra um cenário de
    /// "reiniciar o node" onde o código de restart poderia, por segurança, chamar `shutdown()`
    /// mais de uma vez antes de reconstruir.
    #[tokio::test]
    async fn shutdown_can_be_called_more_than_once_without_hanging() {
        let node = build_node().await;
        assert!(node.shutdown().await.is_ok());

        let second =
            tokio::time::timeout(std::time::Duration::from_millis(200), node.shutdown()).await;
        assert!(second.is_ok(), "segundo shutdown não deveria travar");
        assert!(second.unwrap().is_ok());
    }

    #[tokio::test]
    async fn initial_mode_is_local() {
        let node = build_node().await;
        assert_eq!(node.mode().await, NetworkMode::Local);
    }

    #[tokio::test]
    async fn two_nodes_have_distinct_ids() {
        let (emit_a, _) = capture_emitter();
        let (emit_b, _) = capture_emitter();

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

        assert_ne!(node_a.local_id(), node_b.local_id());
    }

    #[tokio::test]
    async fn local_device_id_populated() {
        let node = build_node().await;
        assert!(node.local_device_id().is_some());
    }

    #[tokio::test]
    async fn same_seed_generates_same_device_id() {
        let seed = [0x42u8; 32];

        let node_a = AcerolaP2p::builder(
            no_op_emitter(),
            IrohTransportBuilder::default().seed(seed),
            test_device_info(),
        )
        .build()
        .await
        .unwrap();

        let node_b = AcerolaP2p::builder(
            no_op_emitter(),
            IrohTransportBuilder::default().seed(seed),
            test_device_info(),
        )
        .build()
        .await
        .unwrap();

        assert_eq!(node_a.local_device_id(), node_b.local_device_id());
    }

    #[tokio::test]
    async fn different_seeds_generate_different_device_ids() {
        let node_a = AcerolaP2p::builder(
            no_op_emitter(),
            IrohTransportBuilder::default().seed([0x11u8; 32]),
            test_device_info(),
        )
        .build()
        .await
        .unwrap();

        let node_b = AcerolaP2p::builder(
            no_op_emitter(),
            IrohTransportBuilder::default().seed([0x22u8; 32]),
            test_device_info(),
        )
        .build()
        .await
        .unwrap();

        assert_ne!(node_a.local_device_id(), node_b.local_device_id());
    }

    #[tokio::test]
    async fn device_info_name_accessible_after_build() {
        let info = DeviceInfo {
            name: "my-pc".to_string(),
            os: "linux".to_string(),
            version: "1.0.0".to_string(),
        };
        let node = AcerolaP2p::builder(no_op_emitter(), IrohTransportBuilder::default(), info)
            .build()
            .await
            .unwrap();
        assert_eq!(node.local_device_info().await.name, "my-pc");
    }

    #[tokio::test]
    async fn device_info_os_accessible_after_build() {
        let info = DeviceInfo {
            name: "my-pc".to_string(),
            os: "windows".to_string(),
            version: "1.0.0".to_string(),
        };
        let node = AcerolaP2p::builder(no_op_emitter(), IrohTransportBuilder::default(), info)
            .build()
            .await
            .unwrap();
        assert_eq!(node.local_device_info().await.os, "windows");
    }

    #[tokio::test]
    async fn device_info_version_accessible_after_build() {
        let info = DeviceInfo {
            name: "my-pc".to_string(),
            os: "linux".to_string(),
            version: "2.3.1".to_string(),
        };
        let node = AcerolaP2p::builder(no_op_emitter(), IrohTransportBuilder::default(), info)
            .build()
            .await
            .unwrap();
        assert_eq!(node.local_device_info().await.version, "2.3.1");
    }

    #[tokio::test]
    async fn two_nodes_with_distinct_device_infos_are_independent() {
        let (emit_a, _) = capture_emitter();
        let (emit_b, _) = capture_emitter();

        let node_a = AcerolaP2p::builder(
            emit_a,
            IrohTransportBuilder::default(),
            DeviceInfo {
                name: "desktop".to_string(),
                os: "linux".to_string(),
                version: "1.0.0".to_string(),
            },
        )
        .build()
        .await
        .unwrap();

        let node_b = AcerolaP2p::builder(
            emit_b,
            IrohTransportBuilder::default(),
            DeviceInfo {
                name: "android".to_string(),
                os: "android".to_string(),
                version: "1.0.0".to_string(),
            },
        )
        .build()
        .await
        .unwrap();

        assert_ne!(node_a.local_device_info().await.name, node_b.local_device_info().await.name);
        assert_ne!(node_a.local_device_info().await.os, node_b.local_device_info().await.os);
    }

    #[tokio::test]
    async fn set_local_device_name_is_visible_immediately_without_rebuilding() {
        let node = AcerolaP2p::builder(
            no_op_emitter(),
            IrohTransportBuilder::default(),
            DeviceInfo {
                name: "old-name".to_string(),
                os: "linux".to_string(),
                version: "1.0.0".to_string(),
            },
        )
        .build()
        .await
        .unwrap();

        assert_eq!(node.local_device_info().await.name, "old-name");

        node.set_local_device_name("new-name".to_string()).await;

        assert_eq!(node.local_device_info().await.name, "new-name");
    }

    #[tokio::test]
    async fn connected_peers_with_info_returns_peers_and_device_info() {
        let node = build_node().await;
        let peer = PeerId { id: "test-connected-peer".to_string(), device_id: None };
        let addr = PeerAddr { id: peer.clone(), addrs: vec![] };
        let device_info = test_device_info();

        {
            let mut state_guard = node.state.write().await;
            state_guard.connect(peer.clone(), addr, b"acerola/handshake/1".to_vec());
            state_guard.store_device_info(peer.clone(), device_info.clone());
        }

        let peers_with_info = node.connected_peers_with_info().await;
        assert_eq!(peers_with_info.len(), 1);
        assert_eq!(peers_with_info[0].0, peer);
        assert!(peers_with_info[0].1.contains(b"acerola/handshake/1".as_slice()));
        assert_eq!(peers_with_info[0].2, Some(device_info));
    }

    #[tokio::test]
    async fn known_peers_reflects_peer_seen_via_handshake() {
        let node = build_node().await;
        let peer = PeerId { id: "test-known-peer".to_string(), device_id: None };
        let addr = PeerAddr { id: peer.clone(), addrs: vec![] };
        let device_info = test_device_info();

        {
            let mut state_guard = node.state.write().await;
            state_guard.connect(peer.clone(), addr, b"acerola/handshake/1".to_vec());
            state_guard.store_device_info(peer.clone(), device_info.clone());
            // Simula o handshake terminando (retorno rápido de RpcServerHandler/RpcClientHandler)
            state_guard.disconnect(&peer, b"acerola/handshake/1");
        }

        let known = node.known_peers().await;
        assert_eq!(known.len(), 1);
        assert_eq!(known[0].0, peer);
        assert_eq!(known[0].2, Some(device_info));

        // `connected_peers` continua vazio (a sessão de handshake já terminou) — known_peers é
        // a lista que sobrevive a isso.
        assert!(node.connected_peers().await.is_empty());
    }

    #[tokio::test]
    async fn known_peers_starts_empty() {
        let node = build_node().await;
        assert!(node.known_peers().await.is_empty());
    }

    #[tokio::test]
    async fn is_peer_reachable_false_for_unknown_peer() {
        let node = build_node().await;
        let unknown = PeerId { id: "unreachable-peer".to_string(), device_id: None };
        assert!(!node.is_peer_reachable(&unknown).await);
    }

    #[tokio::test]
    async fn is_peer_reachable_true_after_real_connection() {
        let (emit_a, _) = capture_emitter();
        let (emit_b, _) = capture_emitter();

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
        let id_b = PeerId {
            id: node_b.local_id().to_string(),
            device_id: node_b.local_device_id().map(|s| s.to_string()),
        };

        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
        node_a.connect(addr_b, b"acerola/handshake/1").await.unwrap();

        let mut reachable = false;
        for _ in 0..50 {
            if node_a.is_peer_reachable(&id_b).await {
                reachable = true;
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }

        assert!(
            reachable,
            "peer should stay reachable via the physical connection even after the one-shot \
             handshake RPC session ends"
        );
    }

    #[tokio::test]
    async fn switch_guard_command_updates_network_mode_and_returns_ok() {
        let node = build_node().await;
        let deny_validator: BoxedValidator = Box::new(|_ctx| {
            Box::pin(async { Err(ConnectionError::AuthDenied("switch test".into())) })
        });

        let switch_result = node.switch_guard(deny_validator, NetworkMode::Relay).await;
        assert!(switch_result.is_ok());

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        assert_eq!(node.mode().await, NetworkMode::Relay);
    }

    #[tokio::test]
    async fn shutdown_command_sends_shutdown_signal_and_returns_ok() {
        let node = build_node().await;
        let shutdown_result = node.shutdown().await;
        assert!(shutdown_result.is_ok());

        // Após o shutdown, tentar enviar novos comandos pelo canal command_tx deve falhar com erro
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let peer_address =
            PeerAddr { id: PeerId { id: "remote".to_string(), device_id: None }, addrs: vec![] };
        let connect_result = node.connect(peer_address, b"acerola/handshake/1").await;
        assert!(connect_result.is_err());
    }

    #[tokio::test]
    async fn explicit_drop_without_shutdown_cancels_tasks_and_frees_resources() {
        let node = build_node().await;
        let weak_state = Arc::downgrade(&node.state);

        // Confirma que o estado é mantido por referências fortes enquanto a struct existe
        assert!(weak_state.upgrade().is_some());

        // Dropar explicitamente o AcerolaP2p sem chamar .shutdown()
        drop(node);

        // Aguarda pequeno intervalo para permitir que o Tokio runtime processe o encerramento do task
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Verifica que o task do NetworkManager foi encerrado e a referência Arc do estado foi liberada
        assert!(
            weak_state.upgrade().is_none(),
            "State should be destroyed after dropping AcerolaP2p"
        );
    }
}
