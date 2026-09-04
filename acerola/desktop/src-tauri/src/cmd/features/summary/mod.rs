use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter, Runtime, State};

use crate::{
    cmd::events::{
        shared::ErrorPayload,
        summary::{ComicSummaryItem, ComicSummaryPayload},
    },
    core::services::summary::{ChapterCacheKey, ChapterCacheService, ChapterService, HomeService},
    data::repositories::archive::chapter_archive_repo::ChapterSortCriteria,
};

#[tauri::command]
pub async fn get_comic_summary<R: Runtime>(
    search: Option<String>, app: AppHandle<R>, pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    let pool = pool.inner().clone();

    tokio::spawn(async move {
        let service = HomeService::new(pool);

        match service.get_all(search).await {
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

#[tauri::command]
pub async fn get_comic_by_folder_name(
    folder_name: String, pool: State<'_, SqlitePool>,
) -> Result<Option<ComicSummaryItem>, String> {
    let service = HomeService::new(pool.inner().clone());

    match service.get_by_folder_name(&folder_name).await {
        Ok(Some((view, count, meta, bookmark))) => {
            Ok(Some(ComicSummaryItem::from_view(view, count, meta, bookmark)))
        },
        Ok(None) => Ok(None),
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
pub async fn get_comic_chapter_ids(
    comic_directory_fk: String, pool: State<'_, SqlitePool>,
) -> Result<Vec<String>, String> {
    let comic_directory_id =
        comic_directory_fk.parse::<i64>().map_err(|error| error.to_string())?;

    let service = ChapterService::new(pool.inner().clone());

    let ids =
        service.get_all_chapter_ids(comic_directory_id).await.map_err(|error| error.to_string())?;

    Ok(ids.into_iter().map(|id| id.to_string()).collect())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn get_comic_chapters<R: Runtime>(
    comic_directory_fk: String, volume_id: Option<String>, page: i32, page_size: i32,
    sort_by: String, search_query: Option<String>, app: AppHandle<R>, pool: State<'_, SqlitePool>,
    cache: State<'_, ChapterCacheService>,
) -> Result<(), String> {
    let pool = pool.inner().clone();
    tracing::info!(
        "[get_comic_chapters] Called for comic_directory_fk={}, volume_id={:?}, page={}, page_size={}, sort_by={}, search_query={:?}",
        comic_directory_fk,
        volume_id,
        page,
        page_size,
        sort_by,
        search_query
    );

    let comic_directory_id =
        comic_directory_fk.parse::<i64>().map_err(|error| error.to_string())?;
    let volume_id_filter = match volume_id {
        Some(vid) => Some(vid.parse::<i64>().map_err(|error| error.to_string())?),
        None => None,
    };

    let sort_criteria = match sort_by.as_str() {
        "number_asc" => ChapterSortCriteria::NumberAsc,
        "number_desc" => ChapterSortCriteria::NumberDesc,
        "modified_asc" => ChapterSortCriteria::ModifiedAsc,
        "modified_desc" => ChapterSortCriteria::ModifiedDesc,
        _ => ChapterSortCriteria::NumberAsc,
    };

    let cache_key = ChapterCacheKey {
        comic_directory_fk: comic_directory_id,
        volume_id: volume_id_filter,
        sort_by: sort_by.clone(),
        search_query: search_query.clone(),
        page,
        page_size,
    };

    if let Some(cached) = cache.get(&cache_key) {
        tracing::info!(
            "[get_comic_chapters] Cache hit for comic_directory_fk={}, volume_id={:?}",
            comic_directory_id,
            volume_id_filter
        );
        app.emit("comic:chapters", cached).unwrap();
        return Ok(());
    }

    let cache = cache.inner().clone();

    tokio::spawn(async move {
        let service = ChapterService::new(pool);

        let search_ref = search_query.as_deref();

        match service
            .get_comic_chapters(
                comic_directory_id,
                volume_id_filter,
                page,
                page_size,
                sort_criteria,
                search_ref,
            )
            .await
        {
            Ok(data) => {
                tracing::info!(
                    "[get_comic_chapters] Success, emitting comic:chapters with {} items",
                    data.archive.items.len()
                );
                cache.put(cache_key, data.clone());
                app.emit("comic:chapters", data).unwrap();
            },
            Err(err) => {
                tracing::error!("[get_comic_chapters] Error: {:?}", err);
                app.emit("comic:chapters:error", ErrorPayload::from(&err)).unwrap();
            },
        }
    });

    Ok(())
}
