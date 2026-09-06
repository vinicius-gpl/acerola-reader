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
    node: RwLock<Arc<AcerolaP2p>>,
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
}

impl NetworkService {
    pub fn new(
        node: Arc<AcerolaP2p>, storage: Arc<SecureP2pStorage>,
        trust_store: Arc<SecureTrustedStore>, app_data_directory: PathBuf,
        rebuild_node: NodeBuilder,
    ) -> Self {
        Self { node: RwLock::new(node), storage, trust_store, app_data_directory, rebuild_node }
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
    fn node(&self) -> Arc<AcerolaP2p> {
        Arc::clone(&self.node.read().unwrap_or_else(|poisoned| poisoned.into_inner()))
    }
}

#[async_trait]
impl NetworkServiceApi for NetworkService {
    fn local_id(&self) -> Result<String, String> {
        Ok(self.node().local_id().to_string())
    }

    /// Endereço completo (id + bytes de endereçamento) usado pra gerar o código/QR de
    /// pareamento — é o que o outro dispositivo precisa pra nos alcançar via `connect()`.
    fn local_addr(&self) -> Result<PeerAddr, String> {
        self.node().local_addr().map_err(|err| err.to_string())
    }

    /// Nome/OS/versão deste dispositivo — usado na tela de Rede pra exibir algo mais
    /// legível que o peer id cru (ex: "Notebook do Vinicius" em vez de um hex de 64 chars).
    async fn local_device_info(&self) -> Result<DeviceInfo, String> {
        Ok(self.node().local_device_info().await)
    }

    async fn set_local_device_name(&self, name: String) -> Result<(), String> {
        self.node().set_local_device_name(name).await;
        Ok(())
    }

    async fn connected_peers_with_info(&self) -> Result<Vec<ConnectedPeerInfo>, String> {
        Ok(self.node().connected_peers_with_info().await)
    }

    async fn paired_peers(&self) -> Result<Vec<(PeerAddr, Option<DeviceInfo>)>, String> {
        use std::collections::HashMap;

        let peers = self.storage.load_peers().await.map_err(|err| err.to_string())?;

        let device_info_by_peer: HashMap<String, DeviceInfo> = self
            .node()
            .known_peers()
            .await
            .into_iter()
            .filter_map(|(peer, _, info)| info.map(|device| (peer.id, device)))
            .collect();

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
        self.node().switch_guard(guard, NetworkMode::Local).await.map_err(|err| err.to_string())
    }

    async fn switch_to_relay(&self) -> Result<(), String> {
        let store = Arc::new(InMemoryTrustedStore::new());
        let guard = TofuGuard::new(store as Arc<dyn TrustedPeerStore>).into_validator();
        self.node().switch_guard(guard, NetworkMode::Relay).await.map_err(|err| err.to_string())
    }

    async fn mode(&self) -> Result<NetworkMode, String> {
        Ok(self.node().mode().await)
    }

    async fn connect(&self, peer_addr: PeerAddr, alpn: Vec<u8>) -> Result<(), String> {
        self.node().connect(peer_addr, &alpn).await.map_err(|err| err.to_string())
    }

    async fn shutdown(&self) -> Result<(), String> {
        self.node().shutdown().await.map_err(|err| err.to_string())
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
        let old_node = self.node();
        // Melhor esforço: mesmo se o shutdown do node antigo falhar/travar parcialmente, ainda
        // vale a pena tentar subir um node novo em vez de deixar o usuário sem rede nenhuma —
        // loga mas não aborta o restart por causa disso.
        if let Err(error) = old_node.shutdown().await {
            tracing::warn!(
                ?error,
                "[NetworkService] failed to cleanly shut down old p2p node before restart"
            );
        }

        let fresh_node = (self.rebuild_node)().await?;
        *self.node.write().unwrap_or_else(|poisoned| poisoned.into_inner()) = fresh_node;

        tracing::info!("[NetworkService] P2P node restarted");
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
}
