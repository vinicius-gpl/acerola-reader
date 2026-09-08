use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use acerola_p2p::api::{
    blobs::IrohBlobsConfig,
    guard::{TofuGuard, TrustedPeerStore},
    identity::{DefaultDeviceInfoProvider, DeviceInfoProvider},
    storage::P2PStorage,
    transport::IrohTransportBuilder,
    AcerolaP2p,
};
use tauri::{Emitter, Manager};

use crate::{
    bios::scopes::{read_device_alias_override, read_library_path, read_relay_settings},
    cmd::features::metadata::MetadataState,
    core::services::{
        network::{NetworkService, NetworkServiceApi},
        sync::{file_sync::FileSyncService, history_sync::HistorySyncService},
    },
    data::repositories::sync::SyncHistoryLogRepository,
    infra::{
        error::ComicError,
        security::{
            get_or_create_master_key,
            p2p_storage::{SecureP2pStorage, SharedP2pStorage},
            trusted_store::SecureTrustedStore,
            MasterKeySource,
        },
        sync::protocol::{
            comic_handler::{ComicSyncInbound, ComicSyncOutbound},
            comic_sync_registry::PendingComicSyncRegistry,
            cover_browse_handler::{CoverBrowseInbound, CoverBrowseOutbound},
            cover_request_registry::PendingCoverRequestRegistry,
            file_handler::{FileSyncInbound, FileSyncOutbound},
            file_session_guard::FileSyncSessionGuard,
            history_handler::{HistorySyncInbound, HistorySyncOutbound},
            library_browse_handler::{LibraryBrowseInbound, LibraryBrowseOutbound},
            transfer::{BlobChapterTransfer, ChapterTransfer},
            COMIC_SYNC_ALPN, COVER_BROWSE_ALPN, FILE_SYNC_ALPN, HISTORY_SYNC_ALPN,
            LIBRARY_BROWSE_ALPN,
        },
    },
};

/// Nome do antigo arquivo de seed em texto puro — só é lido uma vez pra migrar pra
/// `identity.enc` (ver [`migrate_plaintext_seed_if_present`]); depois disso deixa de existir.
const LEGACY_PLAINTEXT_SEED_FILE_NAME: &str = "p2p-seed.key";

/// Se `p2p-seed.key` (formato antigo, texto puro) ainda existir e `identity.enc` (formato
/// novo, criptografado) ainda não existir, migra o seed pro storage seguro e apaga o
/// arquivo antigo. Sem isso, quem já tinha pareado dispositivos perderia a identidade
/// P2P atual no primeiro update e precisaria re-parear tudo do zero.
async fn migrate_plaintext_seed_if_present(
    app_data_directory: &Path, secure_storage: &SecureP2pStorage,
) -> Result<(), ComicError> {
    let legacy_path = app_data_directory.join(LEGACY_PLAINTEXT_SEED_FILE_NAME);
    if !legacy_path.exists() {
        return Ok(());
    }

    if secure_storage.load_identity().await.ok().flatten().is_some() {
        // Já migrado numa execução anterior (ex.: apagar o arquivo antigo falhou) — só limpa.
        std::fs::remove_file(&legacy_path).ok();
        return Ok(());
    }

    let legacy_seed = std::fs::read(&legacy_path).map_err(ComicError::Io)?;
    if legacy_seed.len() != 32 {
        tracing::warn!(
            "[Bios::Network] Legacy p2p-seed.key has unexpected length ({} bytes), ignoring",
            legacy_seed.len()
        );
        return Ok(());
    }

    secure_storage.save_identity(&legacy_seed).await.map_err(|err| {
        ComicError::SystemFailure(format!("failed to migrate legacy p2p seed: {err}"))
    })?;
    std::fs::remove_file(&legacy_path).ok();
    tracing::info!(
        "[Bios::Network] Migrated legacy plaintext p2p-seed.key into encrypted identity.enc"
    );

    Ok(())
}

/// Todo ingrediente precisado pra montar (ou remontar) o `AcerolaP2p` — extraído do corpo de
/// `setup_network` pra ser reusável entre o boot inicial e um `NetworkService::restart()`
/// (troca de relay, ou o botão manual "Reiniciar" na tela de Rede). Cada campo é barato de
/// clonar (`Arc` ou `PathBuf`/tipos que já eram `.clone()`ados no fluxo original) de propósito:
/// [`Self::build`] roda de novo a cada restart, então nada aqui pode ser "consumido" só uma vez.
///
/// `relay_mode`/`device_information` deliberadamente NÃO são campos — são relidos do disco
/// (`settings.json` + cofre) a cada chamada de [`Self::build`], não capturados uma vez no boot.
/// É esse recálculo que faz uma troca de relay/apelido se refletir de verdade num restart, em
/// vez de precisar fechar e reabrir o app inteiro.
struct P2pNodeContext {
    app_data_directory: PathBuf,
    event_emitter: acerola_p2p::api::protocol::EventEmitter,
    trusted_store: Arc<SecureTrustedStore>,
    secure_p2p_storage: Arc<SecureP2pStorage>,
    history_sync_service: HistorySyncService,
    file_sync_service: FileSyncService,
    metadata_service: Arc<crate::core::services::metadata::MetadataService>,
    sync_log_repo: SyncHistoryLogRepository,
    file_sync_session_guard: Arc<FileSyncSessionGuard>,
    pending_comic_sync: Arc<PendingComicSyncRegistry>,
    pending_cover_request: Arc<PendingCoverRequestRegistry>,
    chapter_transfer: Arc<dyn ChapterTransfer>,
    remote_covers_dir: PathBuf,
    blob_context: Arc<crate::infra::sync::blob_context::BlobContext>,
}

impl P2pNodeContext {
    /// Monta um `AcerolaP2p` do zero: relê relay/apelido do disco, registra os mesmos handlers
    /// de sempre (as instâncias em si são reaproveitadas via `Arc`/`.clone()`, só o node por
    /// baixo é novo) e reponta o `BlobContext` compartilhado pro node recém-criado antes de
    /// devolver — sem isso, os handlers de sync-files/sync-comic (que já existem desde o
    /// primeiro boot e nunca são recriados) ficariam presos apontando pro node ANTIGO.
    async fn build(&self) -> Result<Arc<AcerolaP2p>, ComicError> {
        let iroh_services_ticket =
            self.secure_p2p_storage.load_iroh_services_ticket().await.unwrap_or_else(|error| {
                tracing::warn!("[Bios::Network] Failed to load Iroh Services ticket: {}", error);
                None
            });
        let relay_mode =
            read_relay_settings(&self.app_data_directory).resolve(iroh_services_ticket.as_deref());
        tracing::info!("[Bios::Network] Using relay mode: {:?}", relay_mode);

        // MITIGAÇÃO TEMPORÁRIA: `IrohBlobsConfig::fs(...)` trava dentro de
        // `FsStore::load_with_opts` (iroh-blobs) — ver comentário histórico em
        // `acerola-p2p/src/core/blobs/iroh/mod.rs`. `.mem()` não persiste blobs entre
        // reinícios do módulo (nem do app), mas destrava enquanto o hang do FsStore não é
        // corrigido.
        let transport_builder =
            IrohTransportBuilder::default().relay_mode(relay_mode).blobs(IrohBlobsConfig::mem());

        // Sem `.seed(...)` — o builder resolve a identidade sozinho a partir do `.storage(...)`
        // abaixo (carrega o seed salvo em `identity.enc`), então a identidade sobrevive a um
        // restart do módulo normalmente, sem repetir pareamento.
        let mut device_information =
            DefaultDeviceInfoProvider::new("0.0.1-beta").provide().map_err(|device_error| {
                ComicError::SystemFailure(format!("Failed to read device info: {:?}", device_error))
            })?;
        if let Some(alias) = read_device_alias_override(&self.app_data_directory) {
            device_information.name = alias;
        }

        let node = match tokio::time::timeout(
            std::time::Duration::from_secs(10),
            AcerolaP2p::builder(
                Arc::clone(&self.event_emitter),
                transport_builder,
                device_information,
            )
            .guard(
                TofuGuard::new(Arc::clone(&self.trusted_store) as Arc<dyn TrustedPeerStore>)
                    .into_validator(),
            )
            .storage(SharedP2pStorage(Arc::clone(&self.secure_p2p_storage)))
            .inbound(
                HISTORY_SYNC_ALPN,
                Arc::new(HistorySyncInbound::new(
                    Arc::clone(&self.event_emitter),
                    self.history_sync_service.clone(),
                    self.sync_log_repo.clone(),
                )),
            )
            .outbound(
                HISTORY_SYNC_ALPN,
                Arc::new(HistorySyncOutbound::new(
                    Arc::clone(&self.event_emitter),
                    self.history_sync_service.clone(),
                    self.sync_log_repo.clone(),
                )),
            )
            .inbound(
                FILE_SYNC_ALPN,
                Arc::new(FileSyncInbound::new(
                    Arc::clone(&self.event_emitter),
                    self.file_sync_service.clone(),
                    Arc::clone(&self.metadata_service),
                    self.sync_log_repo.clone(),
                    Arc::clone(&self.file_sync_session_guard),
                    Arc::clone(&self.chapter_transfer),
                )),
            )
            .outbound(
                FILE_SYNC_ALPN,
                Arc::new(FileSyncOutbound::new(
                    Arc::clone(&self.event_emitter),
                    self.file_sync_service.clone(),
                    Arc::clone(&self.metadata_service),
                    self.sync_log_repo.clone(),
                    Arc::clone(&self.file_sync_session_guard),
                    Arc::clone(&self.chapter_transfer),
                )),
            )
            .inbound(
                COMIC_SYNC_ALPN,
                Arc::new(ComicSyncInbound::new(
                    Arc::clone(&self.event_emitter),
                    self.file_sync_service.clone(),
                    Arc::clone(&self.metadata_service),
                    self.sync_log_repo.clone(),
                    Arc::clone(&self.file_sync_session_guard),
                    Arc::clone(&self.chapter_transfer),
                )),
            )
            .outbound(
                COMIC_SYNC_ALPN,
                Arc::new(ComicSyncOutbound::new(
                    Arc::clone(&self.event_emitter),
                    self.file_sync_service.clone(),
                    Arc::clone(&self.metadata_service),
                    self.sync_log_repo.clone(),
                    Arc::clone(&self.file_sync_session_guard),
                    Arc::clone(&self.pending_comic_sync),
                    Arc::clone(&self.chapter_transfer),
                )),
            )
            .inbound(
                LIBRARY_BROWSE_ALPN,
                Arc::new(LibraryBrowseInbound::new(self.file_sync_service.clone())),
            )
            .outbound(
                LIBRARY_BROWSE_ALPN,
                Arc::new(LibraryBrowseOutbound::new(Arc::clone(&self.event_emitter))),
            )
            .inbound(
                COVER_BROWSE_ALPN,
                Arc::new(CoverBrowseInbound::new(
                    self.file_sync_service.clone(),
                    Arc::clone(&self.chapter_transfer),
                )),
            )
            .outbound(
                COVER_BROWSE_ALPN,
                Arc::new(CoverBrowseOutbound::new(
                    Arc::clone(&self.event_emitter),
                    Arc::clone(&self.chapter_transfer),
                    Arc::clone(&self.pending_cover_request),
                    self.remote_covers_dir.clone(),
                )),
            )
            .build(),
        )
        .await
        {
            Ok(Ok(node_instance)) => node_instance,
            Ok(Err(start_error)) => {
                tracing::error!("[Bios::Network] Failed to start P2P node: {:?}", start_error);
                return Err(ComicError::SystemFailure(format!(
                    "Failed to start p2p node: {:?}",
                    start_error
                )));
            },
            Err(timeout_error) => {
                tracing::error!(
                    "[Bios::Network] Timeout waiting for AcerolaP2p::build(): {:?}",
                    timeout_error
                );
                return Err(ComicError::SystemFailure(
                    "TIMEOUT waiting for AcerolaP2p::build()!".to_string(),
                ));
            },
        };

        let node = Arc::new(node);
        self.blob_context.set_node(&node);
        Ok(node)
    }
}

pub async fn setup_network(app_handle: &tauri::AppHandle) -> Result<(), ComicError> {
    let app_handle_clone = app_handle.clone();

    let event_emitter: acerola_p2p::api::protocol::EventEmitter =
        Arc::new(move |event_name, event_data| {
            app_handle_clone.emit(event_name, event_data).ok();
        });

    let app_data_directory =
        app_handle.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));

    let (master_key, master_key_source) = get_or_create_master_key(&app_data_directory)?;
    // Guardado como estado gerenciado (não só emitido como evento) porque `setup_network`
    // roda numa task assíncrona separada (`bios/mod.rs::setup_runtime`) que pode terminar
    // antes ou depois do frontend montar a tela de Rede — um evento puro seria perdido se
    // emitido antes de qualquer listener existir. `get_security_status` (comando) consulta
    // isso sob demanda; o evento abaixo cobre o caso do frontend já estar ouvindo.
    app_handle.manage(master_key_source);
    if master_key_source == MasterKeySource::FallbackFile {
        // Sem keyring de verdade disponível (ex.: Linux/Hyprland sem gnome-keyring/kwallet
        // rodando) — nunca falha silenciosamente, mesmo princípio do VS Code: avisa o
        // usuário que a chave mestra caiu pro disco sem a proteção extra do SO.
        app_handle.emit("security:keyring_unavailable", ()).ok();
    }

    // Registrado ANTES de abrir trust/peer storage de propósito: `get_sync_history_log` é
    // um recurso independente da identidade/confiança P2P (é só o log local de sessões de
    // sync já ocorridas) — se ele ficasse depois, uma falha real de decrypt em
    // `trusted.enc`/`peers.enc` (`?` abaixo) derrubaria esse comando também como dano
    // colateral, mesmo sem relação nenhuma com o motivo real da falha.
    let database_pool = app_handle.state::<sqlx::SqlitePool>().inner().clone();
    let sync_log_repo = SyncHistoryLogRepository::new(database_pool.clone());
    app_handle.manage(sync_log_repo.clone());

    let trusted_store = Arc::new(
        SecureTrustedStore::open(&app_data_directory, master_key).map_err(ComicError::Io)?,
    );
    let secure_p2p_storage =
        Arc::new(SecureP2pStorage::open(&app_data_directory, master_key).map_err(ComicError::Io)?);
    migrate_plaintext_seed_if_present(&app_data_directory, &secure_p2p_storage).await?;

    // `database_pool`/`sync_log_repo` já resolvidos mais acima, antes da abertura do
    // trust/peer storage.
    //
    // `library_root` (dentro de `file_sync_service`) reavalia o resolver a cada chamada, não
    // uma vez aqui — trocar `library_path` em `settings.json` continua valendo sem precisar
    // reiniciar nada, P2P incluso.
    let remote_covers_dir = app_data_directory.join("remote_covers");

    let history_sync_service = HistorySyncService::new(database_pool.clone());
    let file_sync_service = FileSyncService::new(database_pool, {
        let app_data_directory = app_data_directory.clone();
        move || {
            read_library_path(&app_data_directory)
                .unwrap_or_else(|| app_data_directory.join("library"))
        }
    });
    // Reaproveita a mesma instância já gerenciada por `bios::db::setup_database` (que roda via
    // `block_on` ANTES deste `setup_network` ser sequer disparado — ver `bios/mod.rs`) em vez
    // de construir uma segunda `MetadataService` própria só pra isso: evita duplicar o
    // `reqwest::Client` interno e mantém uma única fonte de verdade pro reprocessamento de
    // `ComicInfo.xml` recebido via sync, seja ele disparado pelo botão manual ou pelo P2P.
    let metadata_service = Arc::clone(&app_handle.state::<MetadataState>().service);
    // Compartilhado entre inbound e outbound: garante que só uma sessão de sync-files por
    // peer rode por vez, nos dois sentidos (ver `file_session_guard.rs`). Também compartilhado
    // com `COMIC_SYNC_ALPN` (mesmo recurso — transferência de arquivos — então uma sessão de
    // biblioteca inteira e uma individual pro mesmo peer não podem rodar ao mesmo tempo). Uma
    // única instância pela vida inteira do processo — sobrevive a um `restart()` do node, não é
    // recriada a cada rebuild (`P2pNodeContext::build` só reaproveita, via `Arc::clone`).
    let file_sync_session_guard = FileSyncSessionGuard::new();

    // Side-channel pro comando Tauri `sync_comic` informar qual `comic_name` o
    // `ComicSyncOutbound` deve usar na próxima sessão que ele iniciar pra um dado peer — ver
    // `comic_sync_registry.rs` pro motivo de precisar disso (o `Handler` é um singleton, não
    // recebe parâmetro por chamada de `connect()`). Registrado como managed state (não recriado
    // a cada rebuild) porque comandos Tauri já seguram uma referência fixa a ele.
    let pending_comic_sync = PendingComicSyncRegistry::new();
    app_handle.manage(Arc::clone(&pending_comic_sync));

    let pending_cover_request = PendingCoverRequestRegistry::new();
    app_handle.manage(Arc::clone(&pending_cover_request));

    // Handlers de `sync-files`/`sync-comic` são registrados no builder ANTES do node existir,
    // mas precisam de `node.blobs()`/`node.known_peers()` pra publicar/buscar blobs —
    // `BlobContext` guarda um `Weak<AcerolaP2p>` repontado a cada `.build()` (ver
    // `infra::sync::blob_context`), inclusive num restart — os handlers em si (e o
    // `chapter_transfer` que os referencia) são criados uma única vez aqui e sobrevivem.
    let blob_context = crate::infra::sync::blob_context::BlobContext::new();
    let chapter_transfer: Arc<dyn ChapterTransfer> =
        Arc::new(BlobChapterTransfer::new(Arc::clone(&blob_context)));

    let context = Arc::new(P2pNodeContext {
        app_data_directory: app_data_directory.clone(),
        event_emitter,
        trusted_store,
        secure_p2p_storage,
        history_sync_service,
        file_sync_service,
        metadata_service,
        sync_log_repo,
        file_sync_session_guard,
        pending_comic_sync,
        pending_cover_request,
        chapter_transfer,
        remote_covers_dir,
        blob_context,
    });

    let p2p_node = context.build().await?;

    // `NetworkService::restart()` (troca de relay, ou o botão manual "Reiniciar") chama isto de
    // novo pra montar um node fresco com a mesma identidade/storage/handlers — ver doc de
    // `NodeBuilder`/`P2pNodeContext`.
    let rebuild_context = Arc::clone(&context);
    let rebuild_node: crate::core::services::network::NodeBuilder = Arc::new(move || {
        let context = Arc::clone(&rebuild_context);
        Box::pin(async move { context.build().await.map_err(|err| err.to_string()) })
    });

    let network_service: Arc<dyn NetworkServiceApi> = Arc::new(NetworkService::new(
        p2p_node,
        Arc::clone(&context.secure_p2p_storage),
        Arc::clone(&context.trusted_store),
        app_data_directory,
        rebuild_node,
    ));
    app_handle.manage(network_service);

    tracing::info!("[Bios::Network] P2P network service initialized successfully");

    Ok(())
}
