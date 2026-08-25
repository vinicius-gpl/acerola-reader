use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter, Runtime, State};

use crate::{
    cmd::events::{shared::ErrorPayload, summary::ComicSummaryPayload},
    core::services::{
        comic::ComicService,
        summary::{ChapterCacheService, HomeService},
    },
    data::repositories::views::SortCriteria,
    infra::error::ComicError,
};

/// Comando Tauri para buscar quadrinhos com ordenação e filtros específicos.
#[tauri::command]
pub async fn get_comic_summary_sorted<R: Runtime>(
    sort_by: String, sort_order: String, show_hidden: Option<bool>,
    metadata_source: Option<String>, app: AppHandle<R>, pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    let pool = pool.inner().clone();

    // Parse do criteria de ordenação
    let criteria = match (sort_by.as_str(), sort_order.as_str()) {
        ("title", "asc") => SortCriteria::TitleAsc,
        ("title", "desc") => SortCriteria::TitleDesc,
        ("chapterCount", "asc") => SortCriteria::ChapterCountAsc,
        ("chapterCount", "desc") => SortCriteria::ChapterCountDesc,
        ("lastUpdated", "asc") => SortCriteria::LastUpdatedAsc,
        ("lastUpdated", "desc") => SortCriteria::LastUpdatedDesc,
        _ => SortCriteria::TitleAsc,
    };

    let show_hidden = show_hidden.unwrap_or(false);

    tokio::spawn(async move {
        let service = HomeService::new(pool);

        match service.get_all_sorted(criteria, show_hidden, metadata_source).await {
            Ok((comics, counts, meta_map, bookmark_map)) => app
                .emit(
                    "home:data",
                    ComicSummaryPayload::from(comics, counts, meta_map, bookmark_map),
                )
                .unwrap(),
            Err(err) => app.emit("home:error", ErrorPayload::from(&err)).unwrap(),
        }
    });

    Ok(())
}


/// Comando Tauri para atualizar visibilidade de múltiplos quadrinhos.
#[tauri::command]
pub async fn update_comics_visibility(
    ids: Vec<i64>, hidden: bool, pool: State<'_, SqlitePool>,
) -> Result<usize, ErrorPayload> {
    let service = ComicService::new(pool.inner().clone());
    service
        .update_hidden_status_batch(&ids, hidden)
        .await
        .map_err(|error| ErrorPayload::from(&error))
}

/// Comando Tauri para deletar múltiplos quadrinhos.
#[tauri::command]
pub async fn delete_comics(
    ids: Vec<i64>, pool: State<'_, SqlitePool>,
) -> Result<usize, ErrorPayload> {
    let service = ComicService::new(pool.inner().clone());
    service.delete_batch(&ids).await.map_err(|error| ErrorPayload::from(&error))
}

/// Comando Tauri para atualizar status de sincronização externa de um quadrinho.
#[tauri::command]
pub async fn toggle_comic_external_sync(
    id: String, enabled: bool, pool: State<'_, SqlitePool>,
) -> Result<(), ErrorPayload> {
    let parsed_id = id.parse::<i64>().map_err(|err| {
        ErrorPayload::from(&ComicError::SystemFailure(format!("Invalid ID: {}", err)))
    })?;

    let service = ComicService::new(pool.inner().clone());
    service
        .update_external_sync_enabled(parsed_id, enabled)
        .await
        .map(|_| ())
        .map_err(|error| ErrorPayload::from(&error))
}

/// Comando Tauri para reescanear pontualmente um único quadrinho (sync leve, sob demanda).
#[tauri::command]
pub async fn rescan_comic(
    id: String, pool: State<'_, SqlitePool>, chapter_cache: State<'_, ChapterCacheService>,
) -> Result<(), ErrorPayload> {
    let parsed_id = id.parse::<i64>().map_err(|err| {
        ErrorPayload::from(&ComicError::SystemFailure(format!("Invalid ID: {}", err)))
    })?;

    let service = ComicService::new(pool.inner().clone());
    let result = service.rescan_comic(parsed_id).await.map_err(|error| ErrorPayload::from(&error));

    // INFO: Rescan pode adicionar/remover/renomear arquivos de capítulo — o
    // cache de get_comic_chapters (chapter_cache.rs) precisa esquecer esse
    // comic pra não continuar servindo a lista antiga.
    chapter_cache.invalidate_comic(parsed_id);

    result
}

/// Comando Tauri para invalidar e reescanear um único quadrinho do zero (sync profunda).
#[tauri::command]
pub async fn deep_rescan_comic(
    id: String, pool: State<'_, SqlitePool>, chapter_cache: State<'_, ChapterCacheService>,
) -> Result<(), ErrorPayload> {
    let parsed_id = id.parse::<i64>().map_err(|err| {
        ErrorPayload::from(&ComicError::SystemFailure(format!("Invalid ID: {}", err)))
    })?;

    let service = ComicService::new(pool.inner().clone());
    let result =
        service.deep_rescan_comic(parsed_id).await.map_err(|error| ErrorPayload::from(&error));

    chapter_cache.invalidate_comic(parsed_id);

    result
}

/// Comando Tauri para gerar a capa de um quadrinho a partir da página do primeiro capítulo.
#[tauri::command]
pub async fn regenerate_comic_cover(
    id: String, pool: State<'_, SqlitePool>,
) -> Result<(), ErrorPayload> {
    let parsed_id = id.parse::<i64>().map_err(|err| {
        ErrorPayload::from(&ComicError::SystemFailure(format!("Invalid ID: {}", err)))
    })?;

    let service = ComicService::new(pool.inner().clone());
    service.regenerate_cover(parsed_id).await.map(|_| ()).map_err(|error| ErrorPayload::from(&error))
}

/// Comando Tauri para gerar a capa de todos os volumes de um quadrinho a partir da página
/// do primeiro capítulo de cada volume.
#[tauri::command]
pub async fn regenerate_volume_covers(
    id: String, pool: State<'_, SqlitePool>,
) -> Result<(), ErrorPayload> {
    let parsed_id = id.parse::<i64>().map_err(|err| {
        ErrorPayload::from(&ComicError::SystemFailure(format!("Invalid ID: {}", err)))
    })?;

    let service = ComicService::new(pool.inner().clone());
    service
        .regenerate_volume_covers(parsed_id)
        .await
        .map(|_| ())
        .map_err(|error| ErrorPayload::from(&error))
}
