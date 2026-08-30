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
    bios::scopes::{read_device_alias_override, read_library_path, read_relay_url_override},
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
            COMIC_SYNC_ALPN, COVER_BROWSE_ALPN, FILE_SYNC_ALPN, HISTORY_SYNC_ALPN, LIBRARY_BROWSE_ALPN,
        },
    },
};

/// Relay oficial do Acerola — default sempre disponível, sem exigir nenhuma configuração.
pub const DEFAULT_RELAY_URL: &str = "https://relay.acerola-comic.com";

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

    // O relay próprio (`relay.acerola-comic.com`) é o default; usuários avançados podem
    // apontar pra outro relay via a tela de Rede, persistido em `settings.json` como
    // `relay_url`. Só é lido na inicialização — trocar em runtime não é suportado.
    let relay_url =
        read_relay_url_override(&app_data_directory).unwrap_or(DEFAULT_RELAY_URL.to_string());
    tracing::info!("[Bios::Network] Using relay: {}", relay_url);

    // Sem `.seed(...)` aqui — o builder resolve a identidade sozinho a partir do
    // `.storage(...)` abaixo (`acerola_builder.rs::resolve_identity`): carrega o seed
    // salvo em `identity.enc` se existir, ou gera um novo e já persiste criptografado.
    // MITIGAÇÃO TEMPORÁRIA: `IrohBlobsConfig::fs(...)` trava dentro de
    // `FsStore::load_with_opts` (iroh-blobs) — nunca testado com store em disco na lib, só em
    // memória (ver `acerola-p2p/src/core/blobs/iroh/mod.rs`, só `mem_store()` nos testes).
    // Estourava o timeout de 10s do `.build()` abaixo, deixando `network_service` sem
    // `.manage()`. `.mem()` não persiste blobs entre reinícios, mas destrava o app
    // imediatamente enquanto o hang do FsStore é isolado/corrigido.
    let transport_builder = IrohTransportBuilder::default().relay(&relay_url).blobs(IrohBlobsConfig::mem());

    // Apelido custom estilo LocalSend (`device_alias` em `settings.json`) sobrescreve o
    // hostname automático nesta inicialização. Renomear em runtime depois (`set_local_device_name`,
    // exposto via comando Tauri) não precisa reiniciar o node — só precisa dessa releitura aqui
    // pra sobreviver ao PRÓXIMO restart do app.
    let mut device_information =
        DefaultDeviceInfoProvider::new("0.0.1-beta").provide().map_err(|device_error| {
            ComicError::SystemFailure(format!("Failed to read device info: {:?}", device_error))
        })?;
    if let Some(alias) = read_device_alias_override(&app_data_directory) {
        device_information.name = alias;
    }

    // `database_pool`/`sync_log_repo` já resolvidos mais acima, antes da abertura do
    // trust/peer storage.
    //
    // O resolver é reavaliado a cada chamada (não resolvido uma vez aqui) porque
    // `setup_network` roda uma única vez, numa task em background disparada no boot — se
    // `library_root` fosse um `PathBuf` fixo, trocar `library_path` em `settings.json`
    // depois do boot (sem reiniciar o app) faria o sync continuar gravando no destino
    // antigo indefinidamente.
    // Precisa ser resolvido ANTES de `app_data_directory` ser movido pra dentro da closure de
    // `library_root` logo abaixo.
    let remote_covers_dir = app_data_directory.join("remote_covers");

    let history_sync_service = HistorySyncService::new(database_pool.clone());
    let file_sync_service = FileSyncService::new(database_pool, move || {
        read_library_path(&app_data_directory).unwrap_or_else(|| app_data_directory.join("library"))
    });
    // Compartilhado entre inbound e outbound: garante que só uma sessão de sync-files por
    // peer rode por vez, nos dois sentidos (ver `file_session_guard.rs`). Também compartilhado
    // com `COMIC_SYNC_ALPN` (mesmo recurso — transferência de arquivos — então uma sessão de
    // biblioteca inteira e uma individual pro mesmo peer não podem rodar ao mesmo tempo).
    let file_sync_session_guard = FileSyncSessionGuard::new();

    // Side-channel pro comando Tauri `sync_comic` informar qual `comic_name` o
    // `ComicSyncOutbound` deve usar na próxima sessão que ele iniciar pra um dado peer — ver
    // `comic_sync_registry.rs` pro motivo de precisar disso (o `Handler` é um singleton, não
    // recebe parâmetro por chamada de `connect()`).
    let pending_comic_sync = PendingComicSyncRegistry::new();
    app_handle.manage(Arc::clone(&pending_comic_sync));

    let pending_cover_request = PendingCoverRequestRegistry::new();
    app_handle.manage(Arc::clone(&pending_cover_request));

    // Handlers de `sync-files`/`sync-comic` são registrados no builder ANTES do node existir,
    // mas precisam de `node.blobs()`/`node.known_peers()` pra publicar/buscar blobs —
    // `BlobContext` guarda um `Weak<AcerolaP2p>` preenchido só depois de `.build()` (ver
    // `infra::sync::blob_context`).
    let blob_context = crate::infra::sync::blob_context::BlobContext::new();
    let chapter_transfer: Arc<dyn ChapterTransfer> =
        Arc::new(BlobChapterTransfer::new(Arc::clone(&blob_context)));

    let p2p_node = match tokio::time::timeout(
        std::time::Duration::from_secs(10),
        AcerolaP2p::builder(Arc::clone(&event_emitter), transport_builder, device_information)
            .guard(
                TofuGuard::new(Arc::clone(&trusted_store) as Arc<dyn TrustedPeerStore>)
                    .into_validator(),
            )
            .storage(SharedP2pStorage(Arc::clone(&secure_p2p_storage)))
            .inbound(
                HISTORY_SYNC_ALPN,
                Arc::new(HistorySyncInbound::new(
                    Arc::clone(&event_emitter),
                    history_sync_service.clone(),
                    sync_log_repo.clone(),
                )),
            )
            .outbound(
                HISTORY_SYNC_ALPN,
                Arc::new(HistorySyncOutbound::new(
                    Arc::clone(&event_emitter),
                    history_sync_service,
                    sync_log_repo.clone(),
                )),
            )
            .inbound(
                FILE_SYNC_ALPN,
                Arc::new(FileSyncInbound::new(
                    Arc::clone(&event_emitter),
                    file_sync_service.clone(),
                    sync_log_repo.clone(),
                    Arc::clone(&file_sync_session_guard),
                    Arc::clone(&chapter_transfer),
                )),
            )
            .outbound(
                FILE_SYNC_ALPN,
                Arc::new(FileSyncOutbound::new(
                    Arc::clone(&event_emitter),
                    file_sync_service.clone(),
                    sync_log_repo.clone(),
                    Arc::clone(&file_sync_session_guard),
                    Arc::clone(&chapter_transfer),
                )),
            )
            .inbound(
                COMIC_SYNC_ALPN,
                Arc::new(ComicSyncInbound::new(
                    Arc::clone(&event_emitter),
                    file_sync_service.clone(),
                    sync_log_repo.clone(),
                    Arc::clone(&file_sync_session_guard),
                    Arc::clone(&chapter_transfer),
                )),
            )
            .outbound(
                COMIC_SYNC_ALPN,
                Arc::new(ComicSyncOutbound::new(
                    Arc::clone(&event_emitter),
                    file_sync_service.clone(),
                    sync_log_repo.clone(),
                    Arc::clone(&file_sync_session_guard),
                    Arc::clone(&pending_comic_sync),
                    Arc::clone(&chapter_transfer),
                )),
            )
            .inbound(LIBRARY_BROWSE_ALPN, Arc::new(LibraryBrowseInbound::new(file_sync_service.clone())))
            .outbound(
                LIBRARY_BROWSE_ALPN,
                Arc::new(LibraryBrowseOutbound::new(Arc::clone(&event_emitter))),
            )
            .inbound(
                COVER_BROWSE_ALPN,
                Arc::new(CoverBrowseInbound::new(file_sync_service, Arc::clone(&chapter_transfer))),
            )
            .outbound(
                COVER_BROWSE_ALPN,
                Arc::new(CoverBrowseOutbound::new(
                    Arc::clone(&event_emitter),
                    Arc::clone(&chapter_transfer),
                    Arc::clone(&pending_cover_request),
                    remote_covers_dir,
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

    let p2p_node = Arc::new(p2p_node);
    blob_context.set_node(&p2p_node);

    let network_service: Arc<dyn NetworkServiceApi> =
        Arc::new(NetworkService::new(p2p_node, secure_p2p_storage, trusted_store));
    app_handle.manage(network_service);

    tracing::info!("[Bios::Network] P2P network service initialized successfully");

    Ok(())
}
