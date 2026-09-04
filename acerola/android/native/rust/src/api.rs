use acerola_p2p::api::AcerolaP2p;
use std::sync::Arc;
use tokio::runtime::Runtime;

#[cfg(target_os = "android")]
use acerola_p2p::api::{
    blobs::IrohBlobsConfig,
    guard::TofuGuard,
    identity::{DefaultDeviceInfoProvider, DeviceInfoProvider},
    network::NetworkMode,
    peer::{PeerAddr, PeerIdentity},
    storage::P2PStorage,
    transport::IrohTransportBuilder,
};

#[cfg(target_os = "android")]
use crate::{
    callbacks::{CoverBrowseProvider, FileSyncProvider, HistorySyncProvider, SecureBlobStore},
    mode::FfiNetworkMode,
    protocol::{
        cover_browse::{
            CoverBrowseInbound, CoverBrowseOutbound, PendingCoverRequestRegistry, COVER_BROWSE_ALPN,
        },
        files::{
            BlobChapterTransfer, ChapterTransfer, ComicSyncInbound, ComicSyncOutbound,
            FileSyncInbound, FileSyncOutbound, FileSyncSessionGuard, PendingComicScope,
            COMIC_SYNC_ALPN, FILE_SYNC_ALPN,
        },
        history::{HistorySyncInbound, HistorySyncOutbound, HISTORY_SYNC_ALPN},
        library_browse::{LibraryBrowseInbound, LibraryBrowseOutbound, LIBRARY_BROWSE_ALPN},
    },
    relay_settings::{FfiRelaySettings, RelayTicketError},
    singleton::TOKIO_RUNTIME,
    storage::{SecureP2pStorage, SharedSecureP2pStorage},
    sync_direction::FfiSyncDirection,
    trust_store::SecureTrustedStore,
};

#[cfg(target_os = "android")]
use std::{collections::HashMap, sync::Mutex};

#[uniffi::export(with_foreign)]
pub trait P2PCallback: Send + Sync {
    fn on_event(&self, event: String, data: String);
}

#[derive(uniffi::Record)]
pub struct FfiPeerAddr {
    pub id: String,
    pub device_id: Option<String>,
    pub addrs: Vec<u8>,
}

/// Item de `get_paired_peers` — igual a `FfiPeerAddr`, mas com `device_name` a mais. Tipo
/// separado (em vez de acrescentar o campo em `FfiPeerAddr`) pra não forçar todo call site
/// que constrói um `FfiPeerAddr` pra discar (`connect`/`sync_comic`/`browse_library`/
/// `browse_cover`) a passar um `device_name` que não faz sentido nesses casos.
#[derive(uniffi::Record)]
pub struct FfiPairedPeer {
    pub id: String,
    pub device_id: Option<String>,
    pub addrs: Vec<u8>,
    /// Vem de `AcerolaP2p::known_peers()` (persiste entre reinícios e sobrevive ao handshake
    /// fechar), não de `connected_peers_with_info()` (só tem dado pros poucos segundos em que
    /// a conexão de handshake está de fato aberta) — era essa a troca errada que deixava
    /// `device_name` quase sempre `None` na UI, caindo no fallback pro peer id cru.
    pub device_name: Option<String>,
}

#[derive(uniffi::Record)]
pub struct FfiConnectedPeer {
    pub peer_id: String,
    pub alpns: Vec<Vec<u8>>,
    pub device_name: Option<String>,
}

#[derive(uniffi::Object)]
pub struct P2PNode {
    node: Arc<AcerolaP2p>,
    runtime: Arc<Runtime>,
    #[cfg(target_os = "android")]
    trust_store: Arc<SecureTrustedStore>,
    #[cfg(target_os = "android")]
    storage: Arc<SecureP2pStorage>,
    /// Ver `protocol::files::PendingComicScope` — estado pendente entre a chamada FFI
    /// `sync_comic` e o `Handler` outbound de `acerola/sync-comic/1` que ela dispara.
    #[cfg(target_os = "android")]
    pending_comic_scope: PendingComicScope,
    /// Ver `protocol::cover_browse::PendingCoverRequestRegistry` — mesma ideia do
    /// `pending_comic_scope`, mas fila FIFO por peer em vez de slot único (o único disparado em
    /// rajada — `SyncViewModel::fetchCoversFor` chama `browse_cover` uma vez por quadrinho da
    /// biblioteca remota, em sequência rápida, pro mesmo peer).
    #[cfg(target_os = "android")]
    pending_cover_scope: Arc<PendingCoverRequestRegistry>,
}

#[uniffi::export]
#[cfg(target_os = "android")]
impl P2PNode {
    #[uniffi::constructor]
    pub fn new(
        callback: Arc<dyn P2PCallback>,
        legacy_data_dir: Option<String>,
        blobs_dir: String,
        relay_settings: FfiRelaySettings,
        device_name: String,
        device_version: String,
        secure_store: Arc<dyn SecureBlobStore>,
        history_provider: Arc<dyn HistorySyncProvider>,
        file_provider: Arc<dyn FileSyncProvider>,
        cover_provider: Arc<dyn CoverBrowseProvider>,
    ) -> Self {
        let runtime = TOKIO_RUNTIME.clone();

        let cb_clone = Arc::clone(&callback);
        let emit: acerola_p2p::api::protocol::EventEmitter = Arc::new(move |event, data| {
            cb_clone.on_event(event.to_string(), data);
        });

        // Só usado pra migrar `identity.seed`/`peers.json`/`trusted.txt`/`blocked.txt` (formato
        // antigo, texto puro, de antes do `SecureBlobStore`) — ver `storage.rs`/`trust_store.rs`.
        let legacy_dir = legacy_data_dir.as_deref().map(std::path::Path::new);

        let trust_store = Arc::new(SecureTrustedStore::open(
            Arc::clone(&secure_store),
            Arc::clone(&emit),
            legacy_dir,
        ));

        let storage = Arc::new(SecureP2pStorage::open(
            Arc::clone(&secure_store),
            legacy_dir,
        ));

        // Ticket da conta do usuário em `services.iroh.computer` (colado por ele na tela de
        // configuração de rede) — vem do MESMO cofre da identidade, não do DataStore. Falha
        // real de backend não deveria travar a inicialização do resto da rede P2P por causa
        // só da rede pública do Iroh, então também vira "sem ticket" aqui, com log.
        let iroh_services_ticket = storage.load_iroh_services_ticket().unwrap_or_else(|error| {
            tracing::warn!(layer = "relay", error = %error, "failed to load Iroh Services ticket");
            None
        });

        let pending_comic_scope: PendingComicScope = Arc::new(Mutex::new(HashMap::new()));
        let pending_cover_scope = PendingCoverRequestRegistry::new();

        // Handlers de `sync-files`/`sync-comic` são registrados no builder ANTES do node
        // existir, mas precisam de `node.blobs()`/`node.known_peers()` pra publicar/buscar
        // blobs — `BlobContext` guarda um `Weak<AcerolaP2p>` preenchido só depois de `.build()`
        // (ver `protocol::blob_context`).
        let blob_context = crate::protocol::blob_context::BlobContext::new();
        let chapter_transfer: Arc<dyn ChapterTransfer> =
            Arc::new(BlobChapterTransfer::new(Arc::clone(&blob_context)));

        let node = {
            let trust_store = Arc::clone(&trust_store);
            let storage_for_builder = Arc::clone(&storage);
            let emit_for_handlers = Arc::clone(&emit);
            let file_sync_guard = Arc::new(FileSyncSessionGuard::default());
            let pending_comic_scope = Arc::clone(&pending_comic_scope);
            let pending_cover_scope = Arc::clone(&pending_cover_scope);
            let chapter_transfer = Arc::clone(&chapter_transfer);
            let cover_provider = Arc::clone(&cover_provider);
            let blob_context = Arc::clone(&blob_context);
            runtime.block_on(async move {
                // MITIGAÇÃO TEMPORÁRIA: `IrohBlobsConfig::fs(blobs_dir)` trava dentro de
                // `FsStore::load_with_opts` (iroh-blobs) — nunca testado com store em disco na
                // lib, só em memória. Travava `runtime.block_on` na THREAD PRINCIPAL do Android
                // (ANR). `.mem()` não persiste blobs entre reinícios, mas destrava o app
                // imediatamente enquanto o hang do FsStore é isolado/corrigido.
                let _ = &blobs_dir;
                let transport = IrohTransportBuilder::default()
                    .relay_mode(relay_settings.resolve(iroh_services_ticket.as_deref()))
                    .blobs(IrohBlobsConfig::mem());

                let device = DefaultDeviceInfoProvider::new(device_name, device_version)
                    .provide()
                    .expect("Failed to read device info");

                let acerola_node = AcerolaP2p::builder(emit, transport, device)
                    .guard(
                        TofuGuard::new(
                            trust_store as Arc<dyn acerola_p2p::api::guard::TrustedPeerStore>,
                        )
                        .into_validator(),
                    )
                    .storage(SharedSecureP2pStorage(storage_for_builder))
                    .inbound(
                        HISTORY_SYNC_ALPN,
                        Arc::new(HistorySyncInbound::new(
                            Arc::clone(&emit_for_handlers),
                            Arc::clone(&history_provider),
                        )),
                    )
                    .outbound(
                        HISTORY_SYNC_ALPN,
                        Arc::new(HistorySyncOutbound::new(
                            Arc::clone(&emit_for_handlers),
                            history_provider,
                        )),
                    )
                    .inbound(
                        FILE_SYNC_ALPN,
                        Arc::new(FileSyncInbound::new(
                            Arc::clone(&emit_for_handlers),
                            Arc::clone(&file_provider),
                            Arc::clone(&file_sync_guard),
                            Arc::clone(&chapter_transfer),
                        )),
                    )
                    .outbound(
                        FILE_SYNC_ALPN,
                        Arc::new(FileSyncOutbound::new(
                            Arc::clone(&emit_for_handlers),
                            Arc::clone(&file_provider),
                            Arc::clone(&file_sync_guard),
                            Arc::clone(&chapter_transfer),
                        )),
                    )
                    // `acerola/sync-comic/1` reaproveita o MESMO `file_sync_guard` do
                    // `sync-files` normal (ver `protocol::files::run_and_report_scoped`) —
                    // as duas ALPNs nunca rodam sessão simultânea pro mesmo peer.
                    .inbound(
                        COMIC_SYNC_ALPN,
                        Arc::new(ComicSyncInbound::new(
                            Arc::clone(&emit_for_handlers),
                            Arc::clone(&file_provider),
                            Arc::clone(&file_sync_guard),
                            Arc::clone(&chapter_transfer),
                        )),
                    )
                    .outbound(
                        COMIC_SYNC_ALPN,
                        Arc::new(ComicSyncOutbound::new(
                            Arc::clone(&emit_for_handlers),
                            Arc::clone(&file_provider),
                            file_sync_guard,
                            Arc::clone(&chapter_transfer),
                            pending_comic_scope,
                        )),
                    )
                    .inbound(
                        LIBRARY_BROWSE_ALPN,
                        Arc::new(LibraryBrowseInbound::new(file_provider)),
                    )
                    .outbound(
                        LIBRARY_BROWSE_ALPN,
                        Arc::new(LibraryBrowseOutbound::new(Arc::clone(&emit_for_handlers))),
                    )
                    .inbound(
                        COVER_BROWSE_ALPN,
                        Arc::new(CoverBrowseInbound::new(
                            Arc::clone(&cover_provider),
                            Arc::clone(&chapter_transfer),
                        )),
                    )
                    .outbound(
                        COVER_BROWSE_ALPN,
                        Arc::new(CoverBrowseOutbound::new(
                            emit_for_handlers,
                            cover_provider,
                            chapter_transfer,
                            pending_cover_scope,
                        )),
                    )
                    .build()
                    .await
                    .expect("Failed to start P2P node");

                let node = Arc::new(acerola_node);
                blob_context.set_node(&node);
                node
            })
        };

        Self {
            node,
            runtime,
            trust_store,
            storage,
            pending_comic_scope,
            pending_cover_scope,
        }
    }

    pub fn get_local_id(&self) -> String {
        self.node.local_id().to_string()
    }

    pub fn get_local_addr(&self) -> FfiPeerAddr {
        let addr = self.node.local_addr();
        FfiPeerAddr {
            id: addr.id.id.clone(),
            device_id: addr.id.device_id.clone(),
            addrs: addr.addrs.clone(),
        }
    }

    /// Repassa ao transporte um aviso de que a rede do SO pode ter mudado (troca de Wi-Fi/dados
    /// móveis, saída de Doze, etc). Existe porque o Android não expõe monitoramento de interface
    /// de rede para código nativo — só para Java/Kotlin via `ConnectivityManager.NetworkCallback`
    /// — então sem essa ponte o transporte QUIC nunca fica sabendo que o caminho de rede mudou e
    /// mantém reaproveitando conexões silenciosamente mortas até o app ser reiniciado. O Kotlin
    /// deve chamar isto a cada `onAvailable`/`onLost`/`onCapabilitiesChanged` do
    /// `NetworkCallback` (ver `P2pService`).
    pub fn notify_network_change(&self) {
        let node = Arc::clone(&self.node);
        self.runtime.spawn(async move {
            node.network_change().await;
        });
    }

    /// Sobrescreve o nome exibido deste dispositivo (apelido custom estilo LocalSend, em vez do
    /// `Build.MODEL` padrão) — vale a partir do próximo handshake, sem precisar reiniciar o
    /// app, já que `AcerolaP2p::set_local_device_name` escreve no mesmo `DeviceInfo`
    /// compartilhado lido pelos handlers de handshake a cada troca de identidade (ver
    /// `lib/p2p/src/api/acerola_p2p.rs`). Persistência entre reinícios é responsabilidade do
    /// lado Kotlin (DataStore), que relê o apelido salvo pra passar como `device_name` no
    /// próximo `P2PNode::new`.
    pub fn set_local_device_name(&self, name: String) {
        let node = Arc::clone(&self.node);
        self.runtime.spawn(async move {
            node.set_local_device_name(name).await;
        });
    }

    pub fn connect(&self, peer_addr: FfiPeerAddr, alpn: Vec<u8>) {
        let node = Arc::clone(&self.node);
        let addr = PeerAddr {
            id: PeerIdentity {
                id: peer_addr.id,
                device_id: peer_addr.device_id,
            },
            addrs: peer_addr.addrs,
        };
        self.runtime.spawn(async move {
            let _ = node.connect(addr, &alpn).await;
        });
    }

    /// Sincroniza um único quadrinho (`comic_name`) com `peer_addr`, na `direction` explícita
    /// escolhida pelo usuário (`Push` = mandar pro peer; `Pull` = puxar dele) — ver
    /// `protocol::files::model::SyncDirection`. Grava `(comic_name, direction)` no registro
    /// pendente ANTES de conectar — é a única forma dessa escolha (que só existe aqui, do lado
    /// que chamou) chegar até `ComicSyncOutbound::handle`, já que `connect()` não carrega
    /// payload (ver `protocol::files::COMIC_SYNC_ALPN`).
    pub fn sync_comic(
        &self,
        peer_addr: FfiPeerAddr,
        comic_name: String,
        direction: FfiSyncDirection,
    ) {
        self.pending_comic_scope
            .lock()
            .expect("pending comic scope mutex poisoned")
            .insert(peer_addr.id.clone(), (comic_name, direction.into()));
        self.connect(peer_addr, COMIC_SYNC_ALPN.to_vec());
    }

    /// Pede a lista de quadrinhos (nome + contagem de capítulos) da biblioteca de `peer_addr`,
    /// sem sincronizar nada — fire-and-forget como `connect()`; o resultado chega depois via
    /// `browse:library:result`/`browse:library:error` (ver `protocol::library_browse`).
    pub fn browse_library(&self, peer_addr: FfiPeerAddr) {
        self.connect(peer_addr, LIBRARY_BROWSE_ALPN.to_vec());
    }

    /// Busca a capa (thumbnail) de `comic_name` na biblioteca de `peer_addr` — `known_version`
    /// é a versão já cacheada localmente (`(peer_id, comic_name, cover_version)`), `None` se
    /// nunca buscou essa capa antes. Mesmo padrão fire-and-forget de `sync_comic`: grava o
    /// escopo pendente antes de conectar, resultado chega via
    /// `browse:cover:result`/`browse:cover:error` (ver `protocol::cover_browse`).
    pub fn browse_cover(
        &self,
        peer_addr: FfiPeerAddr,
        comic_name: String,
        known_version: Option<i64>,
    ) {
        self.pending_cover_scope
            .push(peer_addr.id.clone(), comic_name, known_version);
        self.connect(peer_addr, COVER_BROWSE_ALPN.to_vec());
    }

    pub fn shutdown(&self) {
        let node = Arc::clone(&self.node);
        self.runtime.block_on(async move {
            let _ = node.shutdown().await;
        });
    }

    pub fn switch_to_local(&self) {
        let node = Arc::clone(&self.node);
        let trust_store = Arc::clone(&self.trust_store);
        self.runtime.spawn(async move {
            let guard =
                TofuGuard::new(trust_store as Arc<dyn acerola_p2p::api::guard::TrustedPeerStore>)
                    .into_validator();
            let _ = node.switch_guard(guard, NetworkMode::Local).await;
        });
    }

    pub fn switch_to_relay(&self) {
        let node = Arc::clone(&self.node);
        let trust_store = Arc::clone(&self.trust_store);
        self.runtime.spawn(async move {
            let guard =
                TofuGuard::new(trust_store as Arc<dyn acerola_p2p::api::guard::TrustedPeerStore>)
                    .into_validator();
            let _ = node.switch_guard(guard, NetworkMode::Relay).await;
        });
    }

    pub fn get_mode(&self) -> FfiNetworkMode {
        self.runtime
            .block_on(async { self.node.mode().await.into() })
    }

    pub fn get_connected_peers(&self) -> HashMap<String, Vec<Vec<u8>>> {
        self.runtime.block_on(async {
            self.node
                .connected_peers()
                .await
                .into_iter()
                .map(|(peer, alpns)| (peer.id.clone(), alpns.into_iter().collect()))
                .collect()
        })
    }

    /// Devolve todos os peers já pareados (TOFU) em algum momento, com o último endereço
    /// conhecido — persistem entre reinícios do app e independem de conexão ativa no momento,
    /// já que a conexão de handshake em si dura só alguns segundos (troca PING/PONG/DeviceInfo
    /// e fecha). É essa lista, não `get_connected_peers*`, que deve alimentar "dispositivos
    /// pareados" na UI.
    pub fn get_paired_peers(&self) -> Vec<FfiPairedPeer> {
        let storage = Arc::clone(&self.storage);
        let node = Arc::clone(&self.node);
        self.runtime.block_on(async move {
            let device_names: HashMap<String, String> = node
                .known_peers()
                .await
                .into_iter()
                .filter_map(|(peer, _, info)| info.map(|device| (peer.id.clone(), device.name)))
                .collect();

            storage
                .load_peers()
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|addr| FfiPairedPeer {
                    device_name: device_names.get(&addr.id.id).cloned(),
                    id: addr.id.id,
                    device_id: addr.id.device_id,
                    addrs: addr.addrs,
                })
                .collect()
        })
    }

    /// Desempareia um peer: some da confiança (`trust_store`) e do cache de endereços
    /// conhecidos (`storage`, a mesma fonte de `get_paired_peers`). Não derruba uma conexão
    /// ativa nem bloqueia o peer — se ele tentar se conectar de novo depois, passa pelo mesmo
    /// fluxo TOFU de um dispositivo nunca visto (ver `trust_store::SecureTrustedStore::remove`).
    pub fn remove_paired_peer(&self, id: String) {
        let trust_store = Arc::clone(&self.trust_store);
        let storage = Arc::clone(&self.storage);
        self.runtime.block_on(async move {
            trust_store.remove(&id).await;
            let _ = storage.remove_peer(&id).await;
        });
    }

    pub fn get_connected_peers_with_info(&self) -> Vec<FfiConnectedPeer> {
        self.runtime.block_on(async {
            self.node
                .connected_peers_with_info()
                .await
                .into_iter()
                .map(|(peer, alpns, device)| FfiConnectedPeer {
                    peer_id: peer.id.clone(),
                    alpns: alpns.into_iter().collect(),
                    device_name: device.map(|info| info.name),
                })
                .collect()
        })
    }

    /// `true` se o usuário já colou e salvou um ticket da própria conta em
    /// `services.iroh.computer` — nunca devolve o valor em si (é uma credencial real).
    pub fn has_iroh_services_ticket(&self) -> bool {
        self.storage
            .load_iroh_services_ticket()
            .ok()
            .flatten()
            .is_some()
    }

    /// Valida o formato antes de persistir. Só tem efeito no próximo início do app — a lib não
    /// suporta trocar relay em runtime.
    pub fn set_iroh_services_ticket(&self, ticket: String) -> Result<(), RelayTicketError> {
        let trimmed = ticket.trim();
        acerola_p2p::api::transport::validate_iroh_services_ticket(trimmed).map_err(|error| {
            RelayTicketError::Invalid {
                reason: error.to_string(),
            }
        })?;

        self.storage
            .save_iroh_services_ticket(trimmed)
            .map_err(|error| RelayTicketError::Invalid {
                reason: error.to_string(),
            })
    }

    /// Remove o ticket salvo — usado quando o usuário desliga a fonte ou substitui por um novo.
    pub fn clear_iroh_services_ticket(&self) {
        let _ = self.storage.clear_iroh_services_ticket();
    }
}
