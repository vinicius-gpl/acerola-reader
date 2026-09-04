use std::{path::PathBuf, sync::mpsc};

use anyhow::{Context, Result};
use serde_json::{json, Value};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use tempfile::TempDir;

use super::support::{
    build_webview, invoke_err, invoke_ok, invoke_ok_value, listen_event, recv_event,
};
use crate::{
    cmd::features::{library::comic_scanner_cmd, summary as summary_cmd},
    core::services::{
        archive::comic_scanner_engine::ComicScannerService, summary::ChapterCacheService,
    },
    tests::utils::setup_test_db::setup_test_db,
};

async fn in_memory_db() -> SqlitePool {
    setup_test_db().await
}

async fn empty_in_memory_db() -> Result<SqlitePool> {
    Ok(SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await?)
}

fn build_library_app(
    pool: SqlitePool,
) -> Result<(tauri::App<tauri::test::MockRuntime>, tauri::WebviewWindow<tauri::test::MockRuntime>)>
{
    build_webview(
        tauri::test::mock_builder().manage(pool).manage(ChapterCacheService::new()).invoke_handler(
            tauri::generate_handler![
                comic_scanner_cmd::refresh_library,
                comic_scanner_cmd::incremental_scan,
                comic_scanner_cmd::rebuild_library,
                summary_cmd::get_comic_summary,
                summary_cmd::get_comic_by_folder_name,
                summary_cmd::get_comic_chapters,
            ],
        ),
    )
}

fn create_comic_dir(root: &TempDir, name: &str, chapters: &[&str]) -> Result<PathBuf> {
    let dir = root.path().join(name);
    std::fs::create_dir_all(&dir)?;

    for chapter in chapters {
        std::fs::write(dir.join(chapter), b"fake cbz")?;
    }

    Ok(dir)
}

async fn refresh_direct(pool: &SqlitePool, root: &TempDir) -> Result<()> {
    let service = ComicScannerService::new(root.path().to_path_buf(), pool.clone());
    service.refresh_library(root.path().to_path_buf(), |_| {}, |_| {}).await?;
    Ok(())
}

async fn seed_comic(pool: &SqlitePool, name: &str) -> Result<i64> {
    let next_id: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(id), 0) + 1 FROM comic_directory")
        .fetch_one(pool)
        .await?;
    let path = format!("/fixtures/{name}");

    sqlx::query(
        "INSERT INTO comic_directory
         (id, name, path, cover, banner, last_modified, archive_template_fk, external_sync_enabled, hidden)
         VALUES (?, ?, ?, NULL, NULL, 0, NULL, 0, 0)",
    )
    .bind(next_id)
    .bind(name)
    .bind(path)
    .execute(pool)
    .await?;

    for index in 1..=3_i64 {
        sqlx::query(
            "INSERT INTO chapter_archive
             (id, chapter, path, chapter_sort, is_special, checksum, comic_directory_fk, volume_fk, last_modified)
             VALUES (?, ?, ?, ?, 0, NULL, ?, NULL, 0)",
        )
        .bind(next_id * 100 + index)
        .bind(format!("Cap {index}"))
        .bind(format!("/fixtures/{name}/cap-{index}.cbz"))
        .bind(index.to_string())
        .bind(next_id)
        .execute(pool)
        .await?;
    }

    Ok(next_id)
}

async fn seed_comic_with_volume(pool: &SqlitePool, name: &str) -> Result<(i64, i64)> {
    let comic_id = seed_comic(pool, name).await?;
    let volume_id = comic_id * 10;

    sqlx::query(
        "INSERT INTO volume_archive
         (id, name, path, volume_sort, is_special, cover, banner, comic_directory_fk, last_modified)
         VALUES (?, 'Vol 01', '/fixtures/volume-1', '1', 0, NULL, NULL, ?, 0)",
    )
    .bind(volume_id)
    .bind(comic_id)
    .execute(pool)
    .await?;

    for index in 1..=2_i64 {
        sqlx::query(
            "INSERT INTO chapter_archive
             (id, chapter, path, chapter_sort, is_special, checksum, comic_directory_fk, volume_fk, last_modified)
             VALUES (?, ?, ?, ?, 0, NULL, ?, ?, 0)",
        )
        .bind(comic_id * 1000 + index)
        .bind(format!("Vol Cap {index}"))
        .bind(format!("/fixtures/{name}/vol-1/cap-{index}.cbz"))
        .bind(index.to_string())
        .bind(comic_id)
        .bind(volume_id)
        .execute(pool)
        .await?;
    }

    Ok((comic_id, volume_id))
}

async fn count_comics(pool: &SqlitePool) -> Result<i64> {
    Ok(sqlx::query_scalar("SELECT COUNT(*) FROM comic_directory").fetch_one(pool).await?)
}

async fn count_chapters(pool: &SqlitePool) -> Result<i64> {
    Ok(sqlx::query_scalar("SELECT COUNT(*) FROM chapter_archive").fetch_one(pool).await?)
}

async fn comic_names(pool: &SqlitePool) -> Result<Vec<String>> {
    Ok(sqlx::query_scalar("SELECT name FROM comic_directory ORDER BY name").fetch_all(pool).await?)
}

async fn wait_scan_complete(complete_rx: mpsc::Receiver<String>) -> Result<Value> {
    recv_event(complete_rx, "scan:complete").await
}

#[tokio::test(flavor = "multi_thread")]
async fn test_refresh_library_emits_progress_and_complete() -> Result<()> {
    let pool = in_memory_db().await;
    let (app, webview) = build_library_app(pool.clone())?;
    let root = TempDir::new()?;
    create_comic_dir(&root, "Berserk", &["Ch. 1.cbz", "Ch. 2.cbz"])?;
    let progress_rx = listen_event(&app, "scan:progress");
    let complete_rx = listen_event(&app, "scan:complete");

    let _: Value = invoke_ok(
        &webview,
        "refresh_library",
        json!({ "path": root.path().to_string_lossy().to_string() }),
    )?;

    let progress = recv_event(progress_rx, "scan:progress").await?;
    let complete = wait_scan_complete(complete_rx).await?;

    assert!(progress.as_str().unwrap_or_default().contains("Berserk"));
    assert_eq!(complete, Value::Null);
    assert_eq!(count_comics(&pool).await?, 1);
    assert_eq!(count_chapters(&pool).await?, 2);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_refresh_library_emits_scan_error_for_nonexistent_path() -> Result<()> {
    let pool = in_memory_db().await;
    let (app, webview) = build_library_app(pool)?;
    let root = TempDir::new()?;
    let missing = root.path().join("does-not-exist");
    let error_rx = listen_event(&app, "scan:error");

    let _: Value = invoke_ok(
        &webview,
        "refresh_library",
        json!({ "path": missing.to_string_lossy().to_string() }),
    )?;

    let error = recv_event(error_rx, "scan:error").await?;

    assert!(error["message"].as_str().unwrap_or_default().contains("not found"));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_incremental_scan_ignores_unchanged_comic_and_emits_complete() -> Result<()> {
    let pool = in_memory_db().await;
    let (app, webview) = build_library_app(pool.clone())?;
    let root = TempDir::new()?;
    create_comic_dir(&root, "Berserk", &["Ch. 1.cbz"])?;
    refresh_direct(&pool, &root).await?;
    let progress_rx = listen_event(&app, "scan:progress");
    let complete_rx = listen_event(&app, "scan:complete");

    let _: Value = invoke_ok(
        &webview,
        "incremental_scan",
        json!({ "path": root.path().to_string_lossy().to_string() }),
    )?;

    let complete = wait_scan_complete(complete_rx).await?;

    assert_eq!(complete, Value::Null);
    assert!(matches!(progress_rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
    assert_eq!(count_comics(&pool).await?, 1);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_incremental_scan_adds_new_removes_deleted_and_emits_progress() -> Result<()> {
    let pool = in_memory_db().await;
    let (app, webview) = build_library_app(pool.clone())?;
    let root = TempDir::new()?;
    let berserk_dir = create_comic_dir(&root, "Berserk", &["Ch. 1.cbz"])?;
    refresh_direct(&pool, &root).await?;
    std::fs::remove_dir_all(berserk_dir)?;
    create_comic_dir(&root, "Vinland Saga", &["Ch. 1.cbz"])?;
    let progress_rx = listen_event(&app, "scan:progress");
    let complete_rx = listen_event(&app, "scan:complete");

    let _: Value = invoke_ok(
        &webview,
        "incremental_scan",
        json!({ "path": root.path().to_string_lossy().to_string() }),
    )?;

    let progress = recv_event(progress_rx, "scan:progress").await?;
    let _ = wait_scan_complete(complete_rx).await?;

    assert!(progress.as_str().unwrap_or_default().contains("Vinland Saga"));
    assert_eq!(comic_names(&pool).await?, vec!["Vinland Saga".to_string()]);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_incremental_scan_emits_scan_error_for_nonexistent_path() -> Result<()> {
    let pool = in_memory_db().await;
    let (app, webview) = build_library_app(pool)?;
    let root = TempDir::new()?;
    let missing = root.path().join("does-not-exist");
    let error_rx = listen_event(&app, "scan:error");

    let _: Value = invoke_ok(
        &webview,
        "incremental_scan",
        json!({ "path": missing.to_string_lossy().to_string() }),
    )?;

    let error = recv_event(error_rx, "scan:error").await?;

    assert!(error["message"].as_str().unwrap_or_default().contains("not found"));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_rebuild_library_reprocesses_without_duplicating_chapters_and_emits_complete(
) -> Result<()> {
    let pool = in_memory_db().await;
    let (app, webview) = build_library_app(pool.clone())?;
    let root = TempDir::new()?;
    create_comic_dir(&root, "Berserk", &["Ch. 1.cbz", "Ch. 2.cbz"])?;
    refresh_direct(&pool, &root).await?;
    let before = count_chapters(&pool).await?;
    let complete_rx = listen_event(&app, "scan:complete");

    let _: Value = invoke_ok(
        &webview,
        "rebuild_library",
        json!({ "path": root.path().to_string_lossy().to_string() }),
    )?;

    let complete = wait_scan_complete(complete_rx).await?;

    assert_eq!(complete, Value::Null);
    assert_eq!(count_chapters(&pool).await?, before);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_rebuild_library_emits_scan_error_for_nonexistent_path() -> Result<()> {
    let pool = in_memory_db().await;
    let (app, webview) = build_library_app(pool)?;
    let root = TempDir::new()?;
    let missing = root.path().join("does-not-exist");
    let error_rx = listen_event(&app, "scan:error");

    let _: Value = invoke_ok(
        &webview,
        "rebuild_library",
        json!({ "path": missing.to_string_lossy().to_string() }),
    )?;

    let error = recv_event(error_rx, "scan:error").await?;

    assert!(error["message"].as_str().unwrap_or_default().contains("not found"));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_get_comic_summary_emits_home_data() -> Result<()> {
    let pool = in_memory_db().await;
    seed_comic(&pool, "Berserk").await?;
    let (app, webview) = build_library_app(pool)?;
    let data_rx = listen_event(&app, "home:data");

    let _: Value = invoke_ok(&webview, "get_comic_summary", json!({}))?;

    let data = recv_event(data_rx, "home:data").await?;

    assert_eq!(data["total"], 1);
    assert_eq!(data["comics"][0]["filesystem"]["folderName"], "Berserk");
    assert_eq!(data["comics"][0]["metadata"]["chapterCount"], 3);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_get_comic_summary_emits_home_error() -> Result<()> {
    let pool = empty_in_memory_db().await?;
    let (app, webview) = build_library_app(pool)?;
    let error_rx = listen_event(&app, "home:error");

    let _: Value = invoke_ok(&webview, "get_comic_summary", json!({}))?;

    let error = recv_event(error_rx, "home:error").await?;

    assert!(error["errorType"].as_str().unwrap_or_default().contains("SystemFailure"));
    assert!(error["message"]
        .as_str()
        .unwrap_or_default()
        .contains("System failure while processing the comic: Internal database error"));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_get_comic_by_folder_name_returns_some() -> Result<()> {
    let pool = in_memory_db().await;
    let comic_id = seed_comic(&pool, "Berserk").await?;
    let (_app, webview) = build_library_app(pool)?;

    let comic =
        invoke_ok_value(&webview, "get_comic_by_folder_name", json!({ "folderName": "Berserk" }))?;

    assert_eq!(comic["relations"]["directoryId"], comic_id.to_string());
    assert_eq!(comic["filesystem"]["folderName"], "Berserk");
    assert_eq!(comic["metadata"]["chapterCount"], 3);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_get_comic_by_folder_name_returns_none() -> Result<()> {
    let pool = in_memory_db().await;
    seed_comic(&pool, "Berserk").await?;
    let (_app, webview) = build_library_app(pool)?;

    let comic = invoke_ok_value(
        &webview,
        "get_comic_by_folder_name",
        json!({ "folderName": "Does Not Exist" }),
    )?;

    assert_eq!(comic, Value::Null);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_get_comic_by_folder_name_serializes_error() -> Result<()> {
    let pool = empty_in_memory_db().await?;
    let (_app, webview) = build_library_app(pool)?;

    let error =
        invoke_err(&webview, "get_comic_by_folder_name", json!({ "folderName": "Berserk" }))?;

    assert!(error
        .as_str()
        .unwrap_or_default()
        .contains("System failure while processing the comic: Internal database error"));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_get_comic_chapters_emits_paginated_page() -> Result<()> {
    let pool = in_memory_db().await;
    let comic_id = seed_comic(&pool, "Berserk").await?;
    let (app, webview) = build_library_app(pool)?;
    let chapters_rx = listen_event(&app, "comic:chapters");

    let _: Value = invoke_ok(
        &webview,
        "get_comic_chapters",
        json!({
            "comicDirectoryFk": comic_id.to_string(),
            "volumeId": Value::Null,
            "page": 1,
            "pageSize": 2,
            "sortBy": "number_asc",
            "searchQuery": Value::Null
        }),
    )?;

    let data = recv_event(chapters_rx, "comic:chapters").await?;

    assert_eq!(data["archive"]["page"], 1);
    assert_eq!(data["archive"]["pageSize"], 2);
    assert_eq!(data["archive"]["total"], 3);
    assert_eq!(data["archive"]["items"].as_array().context("items should be an array")?.len(), 1);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_get_comic_chapters_filters_by_volume() -> Result<()> {
    let pool = in_memory_db().await;
    let (comic_id, volume_id) = seed_comic_with_volume(&pool, "Berserk").await?;
    let (app, webview) = build_library_app(pool)?;
    let chapters_rx = listen_event(&app, "comic:chapters");

    let _: Value = invoke_ok(
        &webview,
        "get_comic_chapters",
        json!({
            "comicDirectoryFk": comic_id.to_string(),
            "volumeId": volume_id.to_string(),
            "page": 0,
            "pageSize": 10,
            "sortBy": "number_asc",
            "searchQuery": Value::Null
        }),
    )?;

    let data = recv_event(chapters_rx, "comic:chapters").await?;

    assert_eq!(data["hasVolumeStructure"], true);
    assert_eq!(data["archive"]["total"], 2);
    assert_eq!(data["archive"]["items"].as_array().context("items should be an array")?.len(), 2);
    assert_eq!(data["archive"]["items"][0]["volumeId"], volume_id.to_string());

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_get_comic_chapters_returns_parse_error_for_comic_id() -> Result<()> {
    let pool = in_memory_db().await;
    let (_app, webview) = build_library_app(pool)?;

    let error = invoke_err(
        &webview,
        "get_comic_chapters",
        json!({
            "comicDirectoryFk": "abc",
            "volumeId": Value::Null,
            "page": 0,
            "pageSize": 10,
            "sortBy": "number_asc",
            "searchQuery": Value::Null
        }),
    )?;

    assert!(error.as_str().unwrap_or_default().contains("invalid digit"));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_get_comic_chapters_returns_parse_error_for_volume_id() -> Result<()> {
    let pool = in_memory_db().await;
    let comic_id = seed_comic(&pool, "Berserk").await?;
    let (_app, webview) = build_library_app(pool)?;

    let error = invoke_err(
        &webview,
        "get_comic_chapters",
        json!({
            "comicDirectoryFk": comic_id.to_string(),
            "volumeId": "volume-invalido",
            "page": 0,
            "pageSize": 10,
            "sortBy": "number_asc",
            "searchQuery": Value::Null
        }),
    )?;

    assert!(error.as_str().unwrap_or_default().contains("invalid digit"));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_get_comic_chapters_filters_by_search_query() -> Result<()> {
    let pool = in_memory_db().await;
    let comic_id = seed_comic(&pool, "Berserk").await?;
    let (app, webview) = build_library_app(pool)?;
    let chapters_rx = listen_event(&app, "comic:chapters");

    let _: Value = invoke_ok(
        &webview,
        "get_comic_chapters",
        json!({
            "comicDirectoryFk": comic_id.to_string(),
            "volumeId": Value::Null,
            "page": 0,
            "pageSize": 10,
            "sortBy": "number_asc",
            "searchQuery": "Cap"
        }),
    )?;

    let data = recv_event(chapters_rx, "comic:chapters").await?;

    let items = data["archive"]["items"].as_array().context("items should be an array")?;
    assert!(!items.is_empty());
    for item in items {
        let name = item["name"].as_str().context("name should be a string")?;
        assert!(name.contains("Cap"));
    }

    Ok(())
}
