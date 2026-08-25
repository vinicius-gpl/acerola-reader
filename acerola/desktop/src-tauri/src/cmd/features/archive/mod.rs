use sqlx::SqlitePool;
use tauri::State;

use crate::{
    cmd::events::shared::ErrorPayload,
    core::services::archive::archive_template_service::ArchiveTemplateService,
    data::models::archive::archive_template::{ArchiveTemplate, SortType},
};

/// Comando Tauri para listar todos os templates de nomenclatura (padrão + do usuário).
#[tauri::command]
pub async fn get_archive_templates(
    pool: State<'_, SqlitePool>,
) -> Result<Vec<ArchiveTemplate>, ErrorPayload> {
    let service = ArchiveTemplateService::new(pool.inner().clone());
    service.list_templates().await.map_err(|error| ErrorPayload::from(&error))
}

/// Comando Tauri para criar um template de nomenclatura definido pelo usuário.
#[tauri::command]
pub async fn create_archive_template(
    label: String, pattern: String, sort_type: SortType, pool: State<'_, SqlitePool>,
) -> Result<ArchiveTemplate, ErrorPayload> {
    let service = ArchiveTemplateService::new(pool.inner().clone());
    service.create_template(label, pattern, sort_type).await.map_err(|error| ErrorPayload::from(&error))
}

/// Comando Tauri para remover um template de nomenclatura criado pelo usuário.
/// Templates padrão são rejeitados pelo serviço e nunca chegam a ser removidos.
#[tauri::command]
pub async fn delete_archive_template(id: i64, pool: State<'_, SqlitePool>) -> Result<(), ErrorPayload> {
    let service = ArchiveTemplateService::new(pool.inner().clone());
    service.delete_template(id).await.map_err(|error| ErrorPayload::from(&error))
}
