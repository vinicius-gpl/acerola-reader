use std::path::PathBuf;

use tauri::Manager;

use crate::{
    cmd::features::metadata::MetadataState,
    core::services::{history::HistoryService, metadata::MetadataService},
    data::repositories::metadata::MetadataRepository,
    infra::error::ComicError,
};

pub async fn setup_database(
    app_handle: &tauri::AppHandle, database_path: PathBuf,
) -> Result<(), ComicError> {
    tracing::info!("[Bios::Db] Connecting to SQLite database at {:?}", database_path);

    use sqlx::sqlite::{
        SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous,
    };
    use std::str::FromStr;

    let connection_options = SqliteConnectOptions::from_str(&format!(
        "sqlite:{}?mode=rwc",
        database_path.to_string_lossy()
    ))
    .map_err(|err| ComicError::SystemFailure(err.to_string()))?
    .journal_mode(SqliteJournalMode::Wal)
    .synchronous(SqliteSynchronous::Normal)
    .busy_timeout(std::time::Duration::from_secs(10));

    let database_pool = match tokio::time::timeout(
        std::time::Duration::from_secs(5),
        SqlitePoolOptions::new()
            .max_connections(10)
            .connect_with(connection_options),
    )
    .await
    {
        Ok(Ok(pool_instance)) => pool_instance,
        Ok(Err(connection_error)) => {
            tracing::error!("[Bios::Db] Failed to connect to SQLite database: {:?}", connection_error);
            return Err(ComicError::SystemFailure(format!(
                "Failed to connect to db: {:?}",
                connection_error
            )));
        },
        Err(timeout_error) => {
            tracing::error!("[Bios::Db] Timeout waiting for SqlitePool::connect: {:?}", timeout_error);
            return Err(ComicError::SystemFailure(
                "TIMEOUT waiting for SqlitePool::connect!".to_string(),
            ));
        },
    };

    app_handle.manage(database_pool.clone());
    app_handle.manage(HistoryService::new(database_pool.clone()));
    app_handle.manage(MetadataState {
        service: std::sync::Arc::new(MetadataService::new(database_pool.clone())),
        repo: MetadataRepository::new(database_pool),
    });

    tracing::info!("[Bios::Db] Database and services initialized successfully");

    Ok(())
}
