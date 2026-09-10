use std::{
    collections::HashSet,
    future::Future,
    path::PathBuf,
    pin::Pin,
    sync::{Arc, RwLock},
};

use acerola_p2p::api::{
    guard::{InMemoryTrustedStore, TofuGuard, TrustedPeerStore},
    identity::DeviceInfo,
    network::NetworkMode,
    peer::{PeerAddr, PeerIdentity},
    storage::P2PStorage,
    AcerolaP2p,
};
use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::infra::security::{p2p_storage::SecureP2pStorage, trusted_store::SecureTrustedStore};

pub type ConnectedPeerInfo = (PeerIdentity, HashSet<Vec<u8>>, Option<DeviceInfo>);

/// Reconstrói um `AcerolaP2p` do zero (mesma identidade/storage/handlers, config de relay
/// relida na hora) — fornecido por `bios::network::setup_network`, que é quem tem em mãos
/// todos os ingredientes específicos de protocolo (`HistorySyncInbound`, `FileSyncInbound`,
/// etc.) que `NetworkService` não precisa conhecer. `NetworkService` só sabe "pedir um node
/// novo", nunca como um é montado — mesma separação que já existia entre `bios::network` (monta)
/// e `NetworkService` (opera o que foi montado).
pub type NodeBuilder = Arc<
    dyn Fn() -> Pin<Box<dyn Future<Output = Result<Arc<AcerolaP2p>, String>> + Send>> + Send + Sync,
>;

#[async_trait]
pub trait NetworkServiceApi: Send + Sync + 'static {
    fn local_id(&self) -> Result<String, String>;
    fn local_addr(&self) -> Result<PeerAddr, String>;
    async fn local_device_info(&self) -> Result<DeviceInfo, String>;
    /// Sobrescreve o nome exibido do dispositivo local (apelido custom estilo LocalSend). Vale
    /// a partir do próximo handshake — não precisa reiniciar o app (ver
    /// `AcerolaP2p::set_local_device_name`). Persistir entre reinícios é responsabilidade do
    /// chamador (frontend grava em `settings.json`, `bios::network::setup_network` reaplica no
    /// próximo boot).
    async fn set_local_device_name(&self, name: String) -> Result<(), String>;
    async fn connected_peers_with_info(&self) -> Result<Vec<ConnectedPeerInfo>, String>;
    /// Todo peer já pareado (TOFU) alguma vez, com o último endereço conhecido — persiste
    /// entre reinícios e independe de conexão ativa agora, diferente de
    /// [`NetworkServiceApi::connected_peers_with_info`] (sessão de protocolo, dura só
    /// segundos). É essa lista que deve alimentar "disparar sync com X" na UI, já que o peer
    /// quase nunca está conectado no exato instante em que o usuário clica o botão.
    ///
    /// `DeviceInfo` vem de `AcerolaP2p::known_peers()` (mesmo mecanismo persistente,
    /// sobrevive ao handshake fechar) — não de `connected_peers_with_info`, que só tem dado
    /// nos poucos segundos em que a sessão de handshake está de fato aberta.
    async fn paired_peers(&self) -> Result<Vec<(PeerAddr, Option<DeviceInfo>)>, String>;
    /// Desempareia um peer: some da confiança (TOFU) e do cache de endereços conhecidos —
    /// mesmo par de fontes que [`NetworkServiceApi::paired_peers`] lê. Não derruba uma
    /// conexão ativa nem bloqueia o peer; se ele reconectar depois, passa pelo mesmo fluxo
    /// TOFU de um dispositivo nunca visto (mesmo comportamento do lado Android, ver
    /// `P2PNode::remove_paired_peer`).
    async fn remove_peer(&self, id: String) -> Result<(), String>;
    async fn switch_to_local(&self) -> Result<(), String>;
    async fn switch_to_relay(&self) -> Result<(), String>;
    async fn mode(&self) -> Result<NetworkMode, String>;
    async fn connect(&self, peer_addr: PeerAddr, alpn: Vec<u8>) -> Result<(), String>;
    async fn shutdown(&self) -> Result<(), String>;
    /// `true` se o usuário já colou e salvou um ticket da própria conta em
    /// `services.iroh.computer` — nunca devolve o valor em si (é uma credencial real).
    async fn has_iroh_services_ticket(&self) -> Result<bool, String>;
    /// Valida o formato antes de persistir (rejeita colagem incompleta/errada na hora, em vez
    /// de só falhar depois). Só tem efeito de verdade depois que o frontend chamar
    /// [`Self::apply_relay_settings`] — que agora reinicia o node (ver [`Self::restart`]), não
    /// precisa mais de um boot novo do app inteiro.
    async fn set_iroh_services_ticket(&self, ticket: String) -> Result<(), String>;
    async fn clear_iroh_services_ticket(&self) -> Result<(), String>;
    /// Relê `settings.json` + o ticket do cofre, resolve a config de relay combinável e aplica
    /// ao node JÁ VIVO (`AcerolaP2p::apply_relay_mode`) — sem isso, mudar a config de relay na
    /// UI só tinha efeito no PRÓXIMO boot do app. Chamado pelo frontend depois de qualquer
    /// mudança nas fontes de relay (toggle do Acerola/Iroh, add/remove de URL própria, ticket).
    async fn apply_relay_settings(&self) -> Result<(), String>;
    /// Desliga o node P2P por completo (fecha o `Endpoint` de verdade — ver `AcerolaP2p::shutdown`
    /// — não só derruba o canal de comandos) e reconstrói do zero com a mesma identidade/storage,
    /// relendo a config de relay atual. Estilo LocalSend: qualquer troca de relay reseta o
    /// módulo inteiro em vez de mutar um `Endpoint` já vivo — era exatamente essa mutação ao
    /// vivo (`apply_relay_mode`) que deixava conexões QUIC num estado confuso depois de uma
    /// troca de rede grande o bastante (ex.: relay próprio -> rede pública Iroh): pacotes
    /// "dropping unexpected packet" no log, discagem repetindo timeout sem nunca resolver.
    /// Também exposta como comando pro usuário disparar manualmente ("Reiniciar" na tela de
    /// Rede), não só automaticamente em [`Self::apply_relay_settings`].
    async fn restart(&self) -> Result<(), String>;
}

pub struct NetworkService {
    node: RwLock<Option<Arc<AcerolaP2p>>>,
    /// Mesmo storage passado ao builder (`.storage(...)`) — clonado antes por
    /// `bios::network::setup_network` pra sobreviver aqui como fonte de "peers pareados"
    /// persistidos, já que a lib não devolve o storage de volta depois do `build()`. Tipo
    /// concreto (não `Arc<dyn P2PStorage>`) porque [`Self::remove_peer`] precisa de
    /// `SecureP2pStorage::remove_peer`, que é específico dessa implementação — não faz parte
    /// da trait compartilhada com o lado Android.
    storage: Arc<SecureP2pStorage>,
    /// Mesmo motivo do campo acima, pro lado da confiança (TOFU) — `SecureTrustedStore::remove`
    /// também não faz parte de `TrustedPeerStore`.
    trust_store: Arc<SecureTrustedStore>,
    /// Só pra [`Self::apply_relay_settings`] poder reler `settings.json` sob demanda — mesma
    /// pasta já usada por `bios::network::setup_network` na inicialização.
    app_data_directory: PathBuf,
    /// Fornecido por `bios::network::setup_network` — ver doc de [`NodeBuilder`]. Chamado só
    /// por [`Self::restart`].
    rebuild_node: NodeBuilder,
    /// Serializa [`Self::restart`] — sem isso, duas chamadas sobrepostas (ex.: dois toggles de
    /// relay em sequência na tela de Rede, ou o toggle e o botão manual "Reiniciar" quase
    /// juntos; Tauri não enfileira `invoke`s, cada um roda numa task própria) constroem DOIS
    /// nodes em paralelo com a MESMA identidade persistida — os dois tentam se registrar ao
    /// mesmo tempo no mesmo relay, que aceita só um e derruba o outro silenciosamente
    /// ("Another endpoint connected with the same endpoint id", visto ao vivo). `tokio::sync`
    /// (não `std::sync`) porque o lock precisa continuar seguro através de vários `.await`
    /// (shutdown do node antigo + rebuild assíncrono), não só de uma seção crítica síncrona
    /// como [`Self::node`].
    restart_lock: Mutex<()>,
}

impl NetworkService {
    pub fn new(
        node: Arc<AcerolaP2p>, storage: Arc<SecureP2pStorage>,
        trust_store: Arc<SecureTrustedStore>, app_data_directory: PathBuf,
        rebuild_node: NodeBuilder,
    ) -> Self {
        Self {
            node: RwLock::new(Some(node)),
            storage,
            trust_store,
            app_data_directory,
            rebuild_node,
            restart_lock: Mutex::new(()),
        }
    }

    pub fn new_uninitialized(
        storage: Arc<SecureP2pStorage>, trust_store: Arc<SecureTrustedStore>,
        app_data_directory: PathBuf, rebuild_node: NodeBuilder,
    ) -> Self {
        Self {
            node: RwLock::new(None),
            storage,
            trust_store,
            app_data_directory,
            rebuild_node,
            restart_lock: Mutex::new(()),
        }
    }

    pub fn set_node(&self, node: Arc<AcerolaP2p>) {
        let mut write = self.node.write().unwrap_or_else(|poisoned| poisoned.into_inner());
        *write = Some(node);
    }

    /// Clona o `Arc` atual e libera o lock na hora — cada chamador opera sobre o node que
    /// estava vivo no instante da chamada, sem segurar o lock por toda a duração de uma
    /// operação de rede potencialmente longa (e sem arriscar um deadlock se essa operação,
    /// direta ou indiretamente, acabar chamando de volta um método deste serviço).
    ///
    /// `std::sync::RwLock` (não `tokio::sync::RwLock`) de propósito: o critical section aqui
    /// é só um `clone()`, nunca atravessa um `.await` — usar a versão síncrona evita de vez
    /// `blocking_read()` (que pode entrar em pânico dentro do runtime tokio, exatamente onde
    /// [`Self::local_id`]/[`Self::local_addr`] — os dois métodos SÍNCRONOS da trait — são
    /// chamados) sem precisar tornar essa função `async` só pelos outros métodos.
    fn node(&self) -> Result<Arc<AcerolaP2p>, String> {
        self.node
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .cloned()
            .ok_or_else(|| "P2P network service is initializing".to_string())
    }
}

#[async_trait]
impl NetworkServiceApi for NetworkService {
    fn local_id(&self) -> Result<String, String> {
        Ok(self.node()?.local_id().to_string())
    }

    /// Endereço completo (id + bytes de endereçamento) usado pra gerar o código/QR de
    /// pareamento — é o que o outro dispositivo precisa pra nos alcançar via `connect()`.
    fn local_addr(&self) -> Result<PeerAddr, String> {
        self.node()?.local_addr().map_err(|err| err.to_string())
    }

    /// Nome/OS/versão deste dispositivo — usado na tela de Rede pra exibir algo mais
    /// legível que o peer id cru (ex: "Notebook do Vinicius" em vez de um hex de 64 chars).
    async fn local_device_info(&self) -> Result<DeviceInfo, String> {
        Ok(self.node()?.local_device_info().await)
    }

    async fn set_local_device_name(&self, name: String) -> Result<(), String> {
        self.node()?.set_local_device_name(name).await;
        Ok(())
    }

    async fn connected_peers_with_info(&self) -> Result<Vec<ConnectedPeerInfo>, String> {
        match self.node() {
            Ok(node) => Ok(node.connected_peers_with_info().await),
            Err(_) => Ok(vec![]),
        }
    }

    async fn paired_peers(&self) -> Result<Vec<(PeerAddr, Option<DeviceInfo>)>, String> {
        use std::collections::HashMap;

        let peers = self.storage.load_peers().await.map_err(|err| err.to_string())?;

        let device_info_by_peer: HashMap<String, DeviceInfo> = match self.node() {
            Ok(node) => node
                .known_peers()
                .await
                .into_iter()
                .filter_map(|(peer, _, info)| info.map(|device| (peer.id, device)))
                .collect(),
            Err(_) => HashMap::new(),
        };

        Ok(peers
            .into_iter()
            .map(|addr| {
                let device = device_info_by_peer.get(&addr.id.id).cloned();
                (addr, device)
            })
            .collect())
    }

    async fn remove_peer(&self, id: String) -> Result<(), String> {
        self.trust_store.remove(&id).await.map_err(|err| err.to_string())?;
        self.storage.remove_peer(&id).await.map_err(|err| err.to_string())
    }

    async fn switch_to_local(&self) -> Result<(), String> {
        let store = Arc::new(InMemoryTrustedStore::new());
        let guard = TofuGuard::new(store as Arc<dyn TrustedPeerStore>).into_validator();
        self.node()?.switch_guard(guard, NetworkMode::Local).await.map_err(|err| err.to_string())
    }

    async fn switch_to_relay(&self) -> Result<(), String> {
        let store = Arc::new(InMemoryTrustedStore::new());
        let guard = TofuGuard::new(store as Arc<dyn TrustedPeerStore>).into_validator();
        self.node()?.switch_guard(guard, NetworkMode::Relay).await.map_err(|err| err.to_string())
    }

    async fn mode(&self) -> Result<NetworkMode, String> {
        match self.node() {
            Ok(node) => Ok(node.mode().await),
            Err(_) => Ok(NetworkMode::Local),
        }
    }

    async fn connect(&self, peer_addr: PeerAddr, alpn: Vec<u8>) -> Result<(), String> {
        self.node()?.connect(peer_addr, &alpn).await.map_err(|err| err.to_string())
    }

    async fn shutdown(&self) -> Result<(), String> {
        if let Ok(node) = self.node() {
            node.shutdown().await.map_err(|err| err.to_string())?;
        }
        Ok(())
    }

    async fn has_iroh_services_ticket(&self) -> Result<bool, String> {
        Ok(self.storage.load_iroh_services_ticket().await.map_err(|err| err.to_string())?.is_some())
    }

    async fn set_iroh_services_ticket(&self, ticket: String) -> Result<(), String> {
        let trimmed = ticket.trim();
        acerola_p2p::api::transport::validate_iroh_services_ticket(trimmed)
            .map_err(|err| err.to_string())?;
        self.storage.save_iroh_services_ticket(trimmed).await.map_err(|err| err.to_string())
    }

    async fn clear_iroh_services_ticket(&self) -> Result<(), String> {
        self.storage.clear_iroh_services_ticket().await.map_err(|err| err.to_string())
    }

    /// Antes mutava o `Endpoint` já vivo (`apply_relay_mode`) — agora delega pra
    /// [`Self::restart`], que reseta o módulo inteiro. Ver doc de [`NetworkServiceApi::restart`]
    /// pro porquê: uma troca de relay grande o bastante (ex.: relay próprio -> rede pública
    /// Iroh) deixava conexões QUIC num estado confuso quando só reconfigurada ao vivo.
    async fn apply_relay_settings(&self) -> Result<(), String> {
        self.restart().await
    }

    async fn restart(&self) -> Result<(), String> {
        // Segura o lock pela operação INTEIRA (não só a troca do ponteiro no fim) — uma
        // segunda chamada concorrente espera esta terminar de verdade (node antigo desligado +
        // node novo já registrado no relay) antes de começar a sua. Ver doc de
        // `Self::restart_lock` pro bug real que isso evita.
        let _restart_guard = self.restart_lock.lock().await;

        if let Ok(old_node) = self.node() {
            // Melhor esforço: mesmo se o shutdown do node antigo falhar/travar parcialmente, ainda
            // vale a pena tentar subir um node novo em vez de deixar o usuário sem rede nenhuma —
            // loga mas não aborta o restart por causa disso.
            if let Err(error) = old_node.shutdown().await {
                tracing::warn!(
                    ?error,
                    "[NetworkService] failed to cleanly shut down old p2p node before restart"
                );
            }
        }

        let fresh_node = (self.rebuild_node)().await?;
        *self.node.write().unwrap_or_else(|poisoned| poisoned.into_inner()) =
            Some(Arc::clone(&fresh_node));

        tracing::info!("[NetworkService] P2P node restarted");

        // Sem isso, um restart devolvia o node num estado limpo — identidade e storage
        // preservados, mas ZERO conexões ativas — e nada tentava falar de novo com nenhum peer
        // pareado até o usuário disparar alguma ação manual (browse library, sync, etc). Achado
        // ao vivo: trocar pra um relay que um peer pareado não compartilha corta o alcance por
        // relay pra ele, e o usuário não tinha NENHUM sinal disso além de tentar algo manualmente
        // e ver falhar. `reconnect_known_peers` só ENFILEIRA a tentativa (não espera confirmação
        // — ver doc lá); best-effort: uma falha ao ler `paired_peers` não deve fazer o restart em
        // si (já concluído com sucesso acima) parecer ter falhado.
        //
        // Sem teste dedicado pro branch `Err` abaixo: `SecureP2pStorage::load_peers` só lê um
        // cache em memória (`peers_cache.read().await.clone()`) e não tem, hoje, nenhum jeito
        // de falhar de verdade — o `Result` existe porque é o contrato da trait `P2PStorage`
        // (implementações futuras, ou outra plataforma, podem genuinamente errar aqui), não
        // porque este caminho é alcançável com o storage concreto atual. Tratado mesmo assim
        // (sem `.unwrap()`/`.expect()`, por regra do CONTRIBUTING.md) por respeitar o contrato
        // do tipo, não por cobertura de teste possível agora.
        match self.storage.load_peers().await {
            Ok(known_peers) => {
                acerola_p2p::api::network::reconnect_known_peers(&fresh_node, known_peers).await;
            },
            Err(error) => {
                tracing::warn!(
                    ?error,
                    "[NetworkService] failed to load paired peers for post-restart reconnect"
                );
            },
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use acerola_p2p::api::{identity::DeviceInfo, transport::IrohTransportBuilder};

    use super::*;

    fn no_op_emitter() -> acerola_p2p::api::protocol::EventEmitter {
        Arc::new(|_event: &str, _payload: String| {})
    }

    fn test_device_info() -> DeviceInfo {
        DeviceInfo {
            name: "test-device".to_string(),
            os: "test-os".to_string(),
            version: "0.0.0".to_string(),
        }
    }

    /// Node real (Iroh de verdade, sem `.storage(...)`) — identidade gerada do zero a cada
    /// chamada, o que é exatamente a propriedade que os testes abaixo exploram pra provar que
    /// `restart()` trocou o node de fato, e não só reexecutou algo em cima do mesmo.
    async fn build_test_node() -> Arc<AcerolaP2p> {
        Arc::new(
            AcerolaP2p::builder(
                no_op_emitter(),
                IrohTransportBuilder::default(),
                test_device_info(),
            )
            .build()
            .await
            .unwrap(),
        )
    }

    fn open_storage() -> (Arc<SecureP2pStorage>, Arc<SecureTrustedStore>, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let key = [7u8; 32];
        let storage = Arc::new(SecureP2pStorage::open(dir.path(), key).unwrap());
        let trust = Arc::new(SecureTrustedStore::open(dir.path(), key).unwrap());
        (storage, trust, dir)
    }

    /// Regressão central desta mudança: `restart()` precisa (1) desligar o node antigo de
    /// verdade (`AcerolaP2p::shutdown`, não só derrubar o canal de comandos — ver o fix em
    /// `lib/p2p`), (2) chamar a closure de rebuild fornecida por `bios::network::setup_network`
    /// exatamente uma vez, e (3) fazer qualquer chamada seguinte (`local_id()`) refletir o node
    /// NOVO, não o antigo. Dois nodes Iroh reais (sem `.storage(...)`, identidade sempre nova)
    /// tornam "o id mudou" uma prova direta de que a troca aconteceu de verdade.
    #[tokio::test]
    async fn restart_shuts_down_old_node_and_swaps_in_a_freshly_built_one() {
        let (storage, trust, _dir) = open_storage();
        let old_node = build_test_node().await;
        let old_local_id = old_node.local_id().to_string();

        let rebuild_calls = Arc::new(AtomicUsize::new(0));
        let rebuild_calls_clone = Arc::clone(&rebuild_calls);
        let rebuild_node: NodeBuilder = Arc::new(move || {
            let rebuild_calls = Arc::clone(&rebuild_calls_clone);
            Box::pin(async move {
                rebuild_calls.fetch_add(1, Ordering::SeqCst);
                Ok(build_test_node().await)
            })
        });

        let service =
            NetworkService::new(old_node, storage, trust, std::env::temp_dir(), rebuild_node);

        service.restart().await.expect("restart should succeed");

        assert_eq!(
            rebuild_calls.load(Ordering::SeqCst),
            1,
            "a closure de rebuild deveria ter sido chamada exatamente uma vez"
        );
        let new_local_id = service.local_id().unwrap();
        assert_ne!(
            old_local_id, new_local_id,
            "restart deveria ter trocado pra um node novo (identidade diferente)"
        );
    }

    /// Regressão de ponta a ponta do bug ao vivo: um restart (troca de relay, ou o botão manual)
    /// devolvia um node com identidade/storage preservados mas ZERO conexões ativas — nada
    /// tentava falar de novo com peers já pareados até o usuário disparar alguma ação manual
    /// (browse library, sync, etc). Prova que `restart()` agora reconecta SOZINHO com um peer
    /// real já pareado, sem nenhuma ação além da própria chamada de `restart()`.
    #[tokio::test]
    async fn restart_reconnects_to_previously_paired_peers_without_further_action() {
        let (storage, trust, _dir) = open_storage();
        let old_node = build_test_node().await;

        let peer_node = build_test_node().await;
        let peer_addr = peer_node.local_addr().unwrap();
        let peer_id = PeerIdentity {
            id: peer_node.local_id().to_string(),
            device_id: peer_node.local_device_id().map(|s| s.to_string()),
        };
        storage.save_peer(&peer_addr).await.expect("seeding paired peer should succeed");

        let rebuild_node: NodeBuilder =
            Arc::new(|| Box::pin(async { Ok(build_test_node().await) }));

        let service = NetworkService::new(
            old_node,
            Arc::clone(&storage),
            trust,
            std::env::temp_dir(),
            rebuild_node,
        );

        service.restart().await.expect("restart should succeed");

        let mut reconnected = false;
        for _ in 0..50 {
            let paired = service.paired_peers().await.expect("paired_peers should succeed");
            if paired.iter().any(|(addr, info)| addr.id == peer_id && info.is_some()) {
                reconnected = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }

        assert!(
            reconnected,
            "restart() deveria ter reconectado sozinho com o peer já pareado, sem ação manual \
             do usuário"
        );
    }

    /// Caminho triste: um peer pareado pode ter saído da rede entre sessões (desligou o app,
    /// perdeu conexão) — isso não pode fazer `restart()` falhar nem impedir a reconexão com
    /// OUTRO peer pareado que continua acessível. `reconnect_known_peers` é best-effort por
    /// design (ver doc em `lib/p2p`), mas só um teste no nível de integração (storage real +
    /// dois peers reais, um deles desligado) prova que a composição dos dois lados não quebra
    /// esse contrato.
    #[tokio::test]
    async fn restart_still_succeeds_and_reconnects_reachable_peer_when_another_paired_peer_is_offline(
    ) {
        let (storage, trust, _dir) = open_storage();
        let old_node = build_test_node().await;

        let reachable_peer = build_test_node().await;
        let reachable_addr = reachable_peer.local_addr().unwrap();
        let reachable_id = PeerIdentity {
            id: reachable_peer.local_id().to_string(),
            device_id: reachable_peer.local_device_id().map(|s| s.to_string()),
        };

        let offline_peer = build_test_node().await;
        let offline_addr = offline_peer.local_addr().unwrap();
        offline_peer.shutdown().await.expect("shutdown should succeed");

        storage.save_peer(&reachable_addr).await.expect("seeding reachable peer should succeed");
        storage.save_peer(&offline_addr).await.expect("seeding offline peer should succeed");

        let rebuild_node: NodeBuilder =
            Arc::new(|| Box::pin(async { Ok(build_test_node().await) }));

        let service = NetworkService::new(
            old_node,
            Arc::clone(&storage),
            trust,
            std::env::temp_dir(),
            rebuild_node,
        );

        service.restart().await.expect("restart should succeed mesmo com um peer pareado offline");

        let mut reconnected = false;
        for _ in 0..50 {
            let paired = service.paired_peers().await.expect("paired_peers should succeed");
            if paired.iter().any(|(addr, info)| addr.id == reachable_id && info.is_some()) {
                reconnected = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }

        assert!(
            reconnected,
            "um peer pareado offline no mesmo restart não pode impedir a reconexão com o peer \
             que continua acessível"
        );
    }

    /// Se a closure de rebuild falhar, `restart()` propaga o erro em vez de mascarar um estado
    /// quebrado — o node antigo já foi desligado nesse ponto, então o usuário PRECISA saber que
    /// o restart não completou, não ver "sucesso" e descobrir só quando a rede não responder
    /// mais.
    #[tokio::test]
    async fn restart_propagates_rebuild_failure() {
        let (storage, trust, _dir) = open_storage();
        let old_node = build_test_node().await;

        let rebuild_node: NodeBuilder = Arc::new(|| Box::pin(async { Err("boom".to_string()) }));

        let service =
            NetworkService::new(old_node, storage, trust, std::env::temp_dir(), rebuild_node);

        let result = service.restart().await;
        assert_eq!(result, Err("boom".to_string()));
    }

    /// Regressão do bug ao vivo: duas mudanças de relay disparadas em sequência (dois toggles,
    /// ou o toggle e o botão manual "Reiniciar") cada uma chama `restart()` numa task própria —
    /// Tauri não enfileira `invoke`s. Sem exclusão mútua, as duas reconstruções rodavam em
    /// paralelo com a MESMA identidade persistida, e o relay via dois endpoints concorrentes
    /// tentando se registrar com o mesmo id ("Another endpoint connected with the same
    /// endpoint id" — a mensagem real vista em produção). Prova que `restart()` serializa: a
    /// rebuild closure nunca observa mais de 1 execução concorrente, mesmo com duas chamadas
    /// disparadas ao mesmo tempo via `tokio::join!`.
    #[tokio::test]
    async fn concurrent_restarts_are_serialized_instead_of_racing() {
        let (storage, trust, _dir) = open_storage();
        let old_node = build_test_node().await;

        let concurrent = Arc::new(AtomicUsize::new(0));
        let max_concurrent = Arc::new(AtomicUsize::new(0));
        let rebuild_calls = Arc::new(AtomicUsize::new(0));

        let concurrent_clone = Arc::clone(&concurrent);
        let max_concurrent_clone = Arc::clone(&max_concurrent);
        let rebuild_calls_clone = Arc::clone(&rebuild_calls);
        let rebuild_node: NodeBuilder = Arc::new(move || {
            let concurrent = Arc::clone(&concurrent_clone);
            let max_concurrent = Arc::clone(&max_concurrent_clone);
            let rebuild_calls = Arc::clone(&rebuild_calls_clone);
            Box::pin(async move {
                rebuild_calls.fetch_add(1, Ordering::SeqCst);
                let now_in_flight = concurrent.fetch_add(1, Ordering::SeqCst) + 1;
                max_concurrent.fetch_max(now_in_flight, Ordering::SeqCst);
                // Espaço suficiente pra uma segunda chamada concorrente ENTRAR em `restart()`
                // enquanto esta ainda constrói — se o lock não estiver protegendo a operação
                // inteira, é aqui que as duas rebuilds se sobrepõem.
                tokio::time::sleep(std::time::Duration::from_millis(30)).await;
                concurrent.fetch_sub(1, Ordering::SeqCst);
                Ok(build_test_node().await)
            })
        });

        let service = Arc::new(NetworkService::new(
            old_node,
            storage,
            trust,
            std::env::temp_dir(),
            rebuild_node,
        ));

        let service_a = Arc::clone(&service);
        let service_b = Arc::clone(&service);
        let (result_a, result_b) =
            tokio::join!(async move { service_a.restart().await }, async move {
                service_b.restart().await
            });

        result_a.expect("primeira chamada de restart deveria ter sucesso");
        result_b.expect("segunda chamada de restart deveria ter sucesso, não ser descartada");

        assert_eq!(
            rebuild_calls.load(Ordering::SeqCst),
            2,
            "as duas chamadas deveriam ter reconstruído o node, nenhuma foi ignorada"
        );
        assert_eq!(
            max_concurrent.load(Ordering::SeqCst),
            1,
            "as duas rebuilds rodaram ao mesmo tempo — restart() não está serializando"
        );
    }

    /// Interação entre os dois fixes recentes (mutex serializando `restart()` +
    /// `reconnect_known_peers` disparado no fim dele): a reconexão que a PRIMEIRA chamada
    /// dispara roda contra o node que ELA construiu, mas se uma SEGUNDA chamada concorrente
    /// entrar logo em seguida (esperando a primeira liberar o lock), o node da primeira já foi
    /// desligado por essa segunda chamada antes de qualquer retry em background da reconexão
    /// terminar. Isso não pode gerar pânico nem deixar o peer pareado sem NENHUMA tentativa de
    /// reconexão no final — a reconexão disparada pela chamada que efetivamente venceu por
    /// último (a mais recente a terminar) é a que precisa ter acontecido de verdade.
    #[tokio::test]
    async fn concurrent_restarts_with_a_paired_peer_do_not_panic_and_still_reconnect() {
        let (storage, trust, _dir) = open_storage();
        let old_node = build_test_node().await;

        let peer_node = build_test_node().await;
        let peer_addr = peer_node.local_addr().unwrap();
        let peer_id = PeerIdentity {
            id: peer_node.local_id().to_string(),
            device_id: peer_node.local_device_id().map(|s| s.to_string()),
        };
        storage.save_peer(&peer_addr).await.expect("seeding paired peer should succeed");

        let rebuild_node: NodeBuilder = Arc::new(|| {
            Box::pin(async {
                // Mesmo espaçamento de `concurrent_restarts_are_serialized_instead_of_racing`
                // — garante que a segunda chamada de fato espera a primeira estar em pleno
                // rebuild antes de tentar entrar, exercitando a transição de node ANTES da
                // reconexão da primeira chamada ter qualquer chance de resolver.
                tokio::time::sleep(std::time::Duration::from_millis(30)).await;
                Ok(build_test_node().await)
            })
        });

        let service = Arc::new(NetworkService::new(
            old_node,
            Arc::clone(&storage),
            trust,
            std::env::temp_dir(),
            rebuild_node,
        ));

        let service_a = Arc::clone(&service);
        let service_b = Arc::clone(&service);
        let (result_a, result_b) =
            tokio::join!(async move { service_a.restart().await }, async move {
                service_b.restart().await
            });

        result_a.expect("primeira chamada de restart deveria ter sucesso");
        result_b.expect("segunda chamada de restart deveria ter sucesso");

        let mut reconnected = false;
        for _ in 0..50 {
            let paired = service.paired_peers().await.expect("paired_peers should succeed");
            if paired.iter().any(|(addr, info)| addr.id == peer_id && info.is_some()) {
                reconnected = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }

        assert!(
            reconnected,
            "depois de dois restarts concorrentes, o node final ainda deveria ter reconectado \
             com o peer pareado — sem pânico nem perda silenciosa da tentativa de reconexão"
        );
    }
}
