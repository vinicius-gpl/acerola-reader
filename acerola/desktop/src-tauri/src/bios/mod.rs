pub mod db;
pub mod network;
pub mod scopes;

use std::path::PathBuf;

use tauri::Manager;

use crate::{
    cmd::features::{
        archive as archive_template_cmd, category as category_cmd, comic as comic_cmd,
        history as history_cmd,
        library::{comic_scanner_cmd, select_folder_cmd},
        metadata as metadata_cmd, network as network_cmd, reader as reader_cmd,
        summary as comic_summary_cmd,
    },
    core::services::{
        network::NetworkServiceApi, reader::ReaderService, summary::ChapterCacheService,
    },
    infra::error::ComicError,
    system_cmd,
};

pub fn build() -> tauri::Builder<tauri::Wry> {
    let app_builder = tauri::Builder::default();
    let app_builder = setup_opener(app_builder);
    let app_builder = setup_dialog(app_builder);
    let app_builder = setup_store(app_builder);
    let app_builder = setup_sql(app_builder);
    let app_builder = setup_fs(app_builder);

    app_builder.setup(setup_runtime).invoke_handler(tauri::generate_handler![
        comic_scanner_cmd::incremental_scan,
        comic_summary_cmd::get_comic_summary,
        comic_summary_cmd::get_comic_by_folder_name,
        comic_summary_cmd::get_comic_chapters,
        comic_summary_cmd::get_comic_chapter_ids,
        comic_scanner_cmd::refresh_library,
        comic_scanner_cmd::rebuild_library,
        select_folder_cmd::select_folder,
        network_cmd::get_network_status,
        network_cmd::switch_to_local,
        network_cmd::switch_to_relay,
        network_cmd::connect_to_peer,
        network_cmd::get_local_id,
        network_cmd::get_local_addr,
        network_cmd::get_local_device_info,
        network_cmd::set_local_device_name,
        network_cmd::get_paired_peers,
        network_cmd::remove_paired_peer,
        network_cmd::get_relay_info,
        network_cmd::set_iroh_services_ticket,
        network_cmd::clear_iroh_services_ticket,
        network_cmd::apply_relay_settings,
        network_cmd::restart_p2p,
        network_cmd::sync_history,
        network_cmd::sync_history_entry,
        network_cmd::sync_files,
        network_cmd::sync_all,
        network_cmd::sync_comic,
        network_cmd::query_remote_library,
        network_cmd::query_remote_cover,
        network_cmd::get_sync_history_log,
        network_cmd::clear_sync_history_log,
        network_cmd::get_security_status,
        reader_cmd::reader_open_chapter,
        reader_cmd::reader_load_page,
        reader_cmd::reader_set_current_page,
        reader_cmd::reader_status,
        reader_cmd::reader_close_chapter,
        reader_cmd::reader_prefetch_window,
        history_cmd::history_update_reading,
        history_cmd::history_get_all,
        history_cmd::history_get_comic,
        history_cmd::history_get_read_chapters,
        history_cmd::history_clear,
        history_cmd::history_mark_chapter_read,
        history_cmd::history_unmark_chapter_read,
        history_cmd::history_mark_chapters_read_batch,
        history_cmd::history_unmark_chapters_read_batch,
        system_cmd::get_package_family_name,
        category_cmd::create_category,
        category_cmd::get_categories,
        category_cmd::delete_category,
        category_cmd::assign_category_to_comic,
        category_cmd::remove_category_from_comic,
        category_cmd::get_comic_category,
        category_cmd::get_all_comic_categories,
        comic_cmd::get_comic_summary_sorted,
        comic_cmd::update_comics_visibility,
        comic_cmd::delete_comics,
        comic_cmd::toggle_comic_external_sync,
        comic_cmd::rescan_comic,
        comic_cmd::deep_rescan_comic,
        comic_cmd::regenerate_comic_cover,
        comic_cmd::regenerate_volume_covers,
        system_cmd::open_filesystem_access_settings,
        metadata_cmd::sync_metadata_mangadex,
        metadata_cmd::sync_metadata_anilist,
        metadata_cmd::sync_all_metadata_mangadex,
        metadata_cmd::sync_all_metadata_anilist,
        metadata_cmd::read_comic_info,
        metadata_cmd::sync_metadata_comic_info,
        metadata_cmd::clear_comic_metadata,
        metadata_cmd::clear_comics_metadata_batch,
        archive_template_cmd::get_archive_templates,
        archive_template_cmd::create_archive_template,
        archive_template_cmd::delete_archive_template,
    ])
}

fn setup_opener(app_builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    app_builder.plugin(tauri_plugin_opener::init())
}

fn setup_dialog(app_builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    app_builder.plugin(tauri_plugin_dialog::init())
}

fn setup_fs(app_builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    app_builder.plugin(tauri_plugin_fs::init())
}

fn setup_store(app_builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    app_builder.plugin(tauri_plugin_store::Builder::new().build())
}

fn setup_sql(app_builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    app_builder.plugin(
        tauri_plugin_sql::Builder::new()
            .add_migrations("sqlite:acerola.db", crate::infra::db::get_migrations())
            .build(),
    )
}

fn setup_runtime(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let app_handle: tauri::AppHandle = app.handle().clone();
    let (app_data_directory, database_path, log_directory) = resolve_paths(app);

    tracing::info!("[Bios] App data directory: {:?}", app_handle.path().app_data_dir().ok());
    tracing::info!("[Bios] Database path: {:?}", database_path);
    tracing::info!("[Bios] Log directory: {:?}", log_directory);

    app.handle().plugin(
        tauri_plugin_log::Builder::new()
            .targets([
                tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Folder {
                    path: log_directory,
                    file_name: None,
                }),
                tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
            ])
            .level(tauri_plugin_log::log::LevelFilter::Info)
            .level_for("acerola", tauri_plugin_log::log::LevelFilter::Debug)
            .level_for("acerola_p2p", tauri_plugin_log::log::LevelFilter::Debug)
            .level_for("acerola_lib", tauri_plugin_log::log::LevelFilter::Debug)
            .build(),
    )?;

    app_handle.manage(ReaderService::new());
    app_handle.manage(ChapterCacheService::new());

    let network_services = tauri::async_runtime::block_on(async {
        db::setup_database(&app_handle, database_path).await.map_err(|db_error| {
            tracing::error!("[Bios] Database initialization error: {:?}", db_error);
            db_error
        })?;
        scopes::setup_scopes_from_store(&app_handle, &app_data_directory).await;
        let services = network::setup_network_services(&app_handle).await.map_err(|net_error| {
            tracing::error!("[Bios] Network services setup error: {:?}", net_error);
            net_error
        })?;
        Ok::<network::InitializedNetworkServices, ComicError>(services)
    })?;

    tauri::async_runtime::spawn(async move {
        if let Err(network_error) = network_services.start_node().await {
            tracing::error!("[Bios] P2P Network node build failed: {:?}", network_error);
        }
    });

    tracing::info!("[Bios] Runtime setup completed successfully");

    Ok(())
}

fn resolve_paths(app: &tauri::App) -> (PathBuf, PathBuf, PathBuf) {
    let base_directory = app.path().app_data_dir().expect("Failed to get app_data_dir");
    let logs_directory = base_directory.join("logs");

    std::fs::create_dir_all(&logs_directory).ok();
    (base_directory.clone(), base_directory.join("acerola.db"), logs_directory)
}

/// Desliga o node P2P de forma graciosa antes do processo terminar — chamado no
/// `RunEvent::Exit` do Tauri (ver `lib.rs`). Sem isso, o `Endpoint` do iroh nunca fecha de
/// verdade (o processo simplesmente morre e o SO recolhe os sockets), o relay não recebe um
/// disconnect limpo e continua tratando esta identidade como "conectada" por um tempo depois
/// do processo já ter saído. Isso deixa a PRÓXIMA execução (ex: depois de recompilar durante o
/// desenvolvimento, ou só reabrir o app rápido) sem conseguir se reconectar direito com peers
/// já pareados — mesma classe de erro do relay ("Another endpoint connected with the same
/// endpoint id") já corrigida pra restarts concorrentes dentro do mesmo processo (ver
/// `NetworkService::restart`), mas nunca coberta pro caso de fechar e reabrir o processo
/// inteiro. Best-effort: se o node ainda nem tinha terminado de subir, ou o shutdown falhar,
/// só loga — o processo vai terminar de qualquer forma, não há como impedir isso aqui.
pub fn shutdown_network<R: tauri::Runtime>(app_handle: &tauri::AppHandle<R>) {
    let Some(network_service) = app_handle.try_state::<std::sync::Arc<dyn NetworkServiceApi>>()
    else {
        return;
    };

    if let Err(error) = tauri::async_runtime::block_on(network_service.shutdown()) {
        tracing::warn!("[Bios] Failed to shut down P2P network on exit: {}", error);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };

    use acerola_p2p::api::{
        identity::DeviceInfo,
        network::NetworkMode,
        peer::{PeerAddr, PeerIdentity},
    };
    use async_trait::async_trait;

    use super::*;
    use crate::core::services::network::ConnectedPeerInfo;

    #[derive(Default)]
    struct RecordingNetworkService {
        shutdown_called: AtomicBool,
    }

    #[async_trait]
    impl NetworkServiceApi for RecordingNetworkService {
        fn local_id(&self) -> Result<String, String> {
            Ok("local-id".to_string())
        }

        fn local_addr(&self) -> Result<PeerAddr, String> {
            Ok(PeerAddr { id: PeerIdentity { id: "local-id".to_string(), device_id: None }, addrs: vec![] })
        }

        async fn local_device_info(&self) -> Result<DeviceInfo, String> {
            Ok(DeviceInfo { name: "test".to_string(), os: "test".to_string(), version: "0.0.0".to_string() })
        }

        async fn set_local_device_name(&self, _name: String) -> Result<(), String> {
            Ok(())
        }

        async fn connected_peers_with_info(&self) -> Result<Vec<ConnectedPeerInfo>, String> {
            Ok(vec![])
        }

        async fn paired_peers(&self) -> Result<Vec<(PeerAddr, Option<DeviceInfo>)>, String> {
            Ok(vec![])
        }

        async fn remove_peer(&self, _id: String) -> Result<(), String> {
            Ok(())
        }

        async fn switch_to_local(&self) -> Result<(), String> {
            Ok(())
        }

        async fn switch_to_relay(&self) -> Result<(), String> {
            Ok(())
        }

        async fn mode(&self) -> Result<NetworkMode, String> {
            Ok(NetworkMode::Local)
        }

        async fn connect(&self, _peer_addr: PeerAddr, _alpn: Vec<u8>) -> Result<(), String> {
            Ok(())
        }

        async fn shutdown(&self) -> Result<(), String> {
            self.shutdown_called.store(true, Ordering::SeqCst);
            Ok(())
        }

        async fn has_iroh_services_ticket(&self) -> Result<bool, String> {
            Ok(false)
        }

        async fn set_iroh_services_ticket(&self, _ticket: String) -> Result<(), String> {
            Ok(())
        }

        async fn clear_iroh_services_ticket(&self) -> Result<(), String> {
            Ok(())
        }

        async fn apply_relay_settings(&self) -> Result<(), String> {
            Ok(())
        }

        async fn restart(&self) -> Result<(), String> {
            Ok(())
        }
    }

    #[test]
    fn shutdown_network_shuts_down_the_managed_network_service() {
        let service = Arc::new(RecordingNetworkService::default());
        let managed: Arc<dyn NetworkServiceApi> = Arc::clone(&service) as Arc<dyn NetworkServiceApi>;
        let app = tauri::test::mock_builder()
            .manage(managed)
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .expect("mock app should build");

        shutdown_network(&app.handle().clone());

        assert!(service.shutdown_called.load(Ordering::SeqCst));
    }

    #[test]
    fn shutdown_network_is_a_no_op_when_no_network_service_is_managed() {
        let app = tauri::test::mock_builder()
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .expect("mock app should build");

        // Não deve entrar em pânico mesmo sem `NetworkServiceApi` gerenciado (ex: app fechado
        // bem cedo, antes de `setup_network_services` terminar).
        shutdown_network(&app.handle().clone());
    }
}
