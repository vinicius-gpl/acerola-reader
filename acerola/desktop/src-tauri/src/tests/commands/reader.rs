use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use tempfile::TempDir;
use tokio::time::{sleep, Duration};
use zip::{
    write::{SimpleFileOptions, ZipWriter},
    CompressionMethod,
};

use super::support::{build_webview, invoke_err, invoke_ok, invoke_ok_value};
use crate::{cmd::features::reader as reader_cmd, core::services::reader::ReaderService};

fn build_reader_app(
    manage_reader: bool,
) -> Result<(tauri::App<tauri::test::MockRuntime>, tauri::WebviewWindow<tauri::test::MockRuntime>)>
{
    let builder = tauri::test::mock_builder().invoke_handler(tauri::generate_handler![
        reader_cmd::reader_open_chapter,
        reader_cmd::reader_load_page,
        reader_cmd::reader_set_current_page,
        reader_cmd::reader_status,
        reader_cmd::reader_close_chapter,
        reader_cmd::reader_prefetch_window,
    ]);

    let builder = if manage_reader { builder.manage(ReaderService::new()) } else { builder };

    build_webview(builder)
}

fn create_cbz(dir: &TempDir, name: &str, pages: &[(&str, &[u8])]) -> Result<PathBuf> {
    let path = dir.path().join(name);
    let file = File::create(&path)?;
    let mut archive = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

    for (entry_name, bytes) in pages {
        archive.start_file(*entry_name, options)?;
        archive.write_all(bytes)?;
    }

    archive.finish()?;
    Ok(path)
}

fn chapter(path: &Path) -> Value {
    json!({
        "id": "chapter-1",
        "name": "Capitulo 1",
        "path": path.to_string_lossy().to_string(),
        "chapterSort": "1",
        "volumeId": "volume-1",
        "volumeName": "Volume 1",
        "isSpecial": false,
        "lastModified": 0
    })
}

fn open_chapter(
    webview: &tauri::WebviewWindow<tauri::test::MockRuntime>, path: &Path,
) -> Result<Value> {
    invoke_ok_value(webview, "reader_open_chapter", json!({ "chapter": chapter(path) }))
}

fn cache_keys(status: &Value) -> Result<Vec<usize>> {
    let keys = status
        .get("cacheKeys")
        .and_then(Value::as_array)
        .context("status should contain cacheKeys as an array")?;

    keys.iter()
        .map(|key| {
            key.as_u64()
                .map(|value| value as usize)
                .context("cacheKeys should contain only numbers")
        })
        .collect()
}

async fn wait_for_cache_keys(
    webview: &tauri::WebviewWindow<tauri::test::MockRuntime>, expected: &[usize],
) -> Result<()> {
    let mut expected_sorted = expected.to_vec();
    expected_sorted.sort_unstable();

    for _ in 0..30 {
        let status = invoke_ok_value(webview, "reader_status", json!({}))?;
        let mut actual = cache_keys(&status)?;
        actual.sort_unstable();

        if actual == expected_sorted {
            return Ok(());
        }

        sleep(Duration::from_millis(25)).await;
    }

    bail!("reader cache did not reach the expected keys: {expected_sorted:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_reader_open_chapter_rejects_invalid_format() -> Result<()> {
    let (_app, webview) = build_reader_app(true)?;
    let dir = TempDir::new()?;
    let path = dir.path().join("chapter.txt");
    std::fs::write(&path, "conteudo invalido")?;

    let error = invoke_err(&webview, "reader_open_chapter", json!({ "chapter": chapter(&path) }))?;

    assert!(error.as_str().unwrap_or_default().contains("not supported"));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_reader_loads_page_with_bytes_mime_and_cache_hit() -> Result<()> {
    let (_app, webview) = build_reader_app(true)?;
    let dir = TempDir::new()?;
    let cbz = create_cbz(&dir, "chapter.cbz", &[("001.jpg", &[1, 2, 3])])?;
    open_chapter(&webview, &cbz)?;

    let first =
        invoke_ok_value(&webview, "reader_load_page", json!({ "index": 0, "setCurrent": true }))?;
    let second =
        invoke_ok_value(&webview, "reader_load_page", json!({ "index": 0, "setCurrent": true }))?;

    assert_eq!(first["chapterId"], "chapter-1");
    assert_eq!(first["index"], 0);
    assert_eq!(first["total"], 1);
    assert_eq!(first["mimeType"], "image/jpeg");
    assert_eq!(first["bytes"], json!([1, 2, 3]));
    assert_eq!(first["cacheHit"], false);
    assert_eq!(second["cacheHit"], true);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_reader_load_page_out_of_bounds_returns_error() -> Result<()> {
    let (_app, webview) = build_reader_app(true)?;
    let dir = TempDir::new()?;
    let cbz = create_cbz(&dir, "chapter.cbz", &[("001.jpg", &[1])])?;
    open_chapter(&webview, &cbz)?;

    let error =
        invoke_err(&webview, "reader_load_page", json!({ "index": 1, "setCurrent": true }))?;

    assert!(error.as_str().unwrap_or_default().contains("out of bounds"));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_reader_sets_current_page_and_updates_status() -> Result<()> {
    let (_app, webview) = build_reader_app(true)?;
    let dir = TempDir::new()?;
    let cbz = create_cbz(
        &dir,
        "chapter.cbz",
        &[("001.jpg", &[1]), ("002.jpg", &[2]), ("003.jpg", &[3])],
    )?;
    open_chapter(&webview, &cbz)?;

    let status = invoke_ok_value(&webview, "reader_set_current_page", json!({ "index": 2 }))?;

    assert_eq!(status["isOpen"], true);
    assert_eq!(status["currentPage"], 2);
    assert_eq!(status["pageCount"], 3);
    assert_eq!(status["cacheKeys"], json!([]));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_reader_set_page_without_session_returns_error() -> Result<()> {
    let (_app, webview) = build_reader_app(true)?;

    let error = invoke_err(&webview, "reader_set_current_page", json!({ "index": 0 }))?;

    assert!(error.as_str().unwrap_or_default().contains("no open chapter"));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_reader_status_returns_closed_and_opened() -> Result<()> {
    let (_app, webview) = build_reader_app(true)?;
    let dir = TempDir::new()?;
    let cbz = create_cbz(&dir, "chapter.cbz", &[("001.jpg", &[1])])?;

    let closed = invoke_ok_value(&webview, "reader_status", json!({}))?;
    open_chapter(&webview, &cbz)?;
    let opened = invoke_ok_value(&webview, "reader_status", json!({}))?;

    assert_eq!(closed["isOpen"], false);
    assert_eq!(closed["chapterId"], Value::Null);
    assert_eq!(opened["isOpen"], true);
    assert_eq!(opened["chapterId"], "chapter-1");
    assert_eq!(opened["pageCount"], 1);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_reader_status_without_managed_state_returns_error() -> Result<()> {
    let (_app, webview) = build_reader_app(false)?;

    let error = invoke_err(&webview, "reader_status", json!({}))?;

    assert!(error.as_str().unwrap_or_default().contains("state"));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_reader_close_chapter_clears_session_cache_and_status() -> Result<()> {
    let (_app, webview) = build_reader_app(true)?;
    let dir = TempDir::new()?;
    let cbz = create_cbz(&dir, "chapter.cbz", &[("001.jpg", &[1])])?;
    open_chapter(&webview, &cbz)?;
    let _: Value =
        invoke_ok(&webview, "reader_load_page", json!({ "index": 0, "setCurrent": true }))?;

    let status = invoke_ok_value(&webview, "reader_close_chapter", json!({}))?;

    assert_eq!(status["isOpen"], false);
    assert_eq!(status["chapterId"], Value::Null);
    assert_eq!(status["pageCount"], 0);
    assert_eq!(status["currentPage"], Value::Null);
    assert_eq!(status["cacheKeys"], json!([]));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_reader_close_without_managed_state_returns_error() -> Result<()> {
    let (_app, webview) = build_reader_app(false)?;

    let error = invoke_err(&webview, "reader_close_chapter", json!({}))?;

    assert!(error.as_str().unwrap_or_default().contains("state"));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_reader_prefetch_window_default_and_custom_loads_cache() -> Result<()> {
    let (_app, webview) = build_reader_app(true)?;
    let dir = TempDir::new()?;
    let cbz = create_cbz(
        &dir,
        "chapter.cbz",
        &[
            ("001.jpg", &[1]),
            ("002.jpg", &[2]),
            ("003.jpg", &[3]),
            ("004.jpg", &[4]),
            ("005.jpg", &[5]),
        ],
    )?;
    open_chapter(&webview, &cbz)?;

    let _: Value = invoke_ok(&webview, "reader_prefetch_window", json!({ "center": 2 }))?;
    wait_for_cache_keys(&webview, &[0, 1, 2, 3, 4]).await?;

    let _ = invoke_ok_value(&webview, "reader_close_chapter", json!({}))?;
    open_chapter(&webview, &cbz)?;

    let _: Value =
        invoke_ok(&webview, "reader_prefetch_window", json!({ "center": 2, "radius": 1 }))?;
    wait_for_cache_keys(&webview, &[1, 2, 3]).await?;

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_reader_prefetch_without_managed_state_returns_error() -> Result<()> {
    let (_app, webview) = build_reader_app(false)?;

    let error =
        invoke_err(&webview, "reader_prefetch_window", json!({ "center": 0, "radius": 1 }))?;

    assert!(error.as_str().unwrap_or_default().contains("state"));

    Ok(())
}
