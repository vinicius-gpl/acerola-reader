use std::path::PathBuf;

use serde_json::Value;
use tauri::Manager;
use tauri_plugin_fs::FsExt;

/// Extrai a propriedade `library_path` do conteúdo JSON das configurações.
fn extract_library_path(file_content: &str) -> Option<PathBuf> {
    let json_value: Value = serde_json::from_str(file_content).ok()?;
    let path_str = json_value.get("library_path")?.as_str()?;
    Some(PathBuf::from(path_str))
}

/// Lê `library_path` diretamente de `settings.json`, sem depender do plugin de store
/// estar inicializado. Usado tanto pelo setup de escopos do FS quanto pelo sync P2P de
/// arquivos, que precisa saber onde gravar capítulos recebidos de outro device.
pub fn read_library_path(app_data_directory: &std::path::Path) -> Option<PathBuf> {
    let settings_file_path = app_data_directory.join("settings.json");
    let file_content = std::fs::read_to_string(&settings_file_path).ok()?;
    extract_library_path(&file_content)
}

/// Lê o override opcional de `relay_url` de `settings.json`. Se ausente (caso comum — a
/// maioria dos usuários nunca mexe nisso), o chamador deve cair no relay padrão hardcoded.
/// Só é lido na inicialização: trocar a URL do relay em runtime não é suportado pela lib,
/// só a troca de modo local/relay (já exposta via `switch_to_local`/`switch_to_relay`).
pub fn read_relay_url_override(app_data_directory: &std::path::Path) -> Option<String> {
    let settings_file_path = app_data_directory.join("settings.json");
    let file_content = std::fs::read_to_string(&settings_file_path).ok()?;
    let json_value: Value = serde_json::from_str(&file_content).ok()?;
    let relay_url = json_value.get("relay_url")?.as_str()?.trim().to_string();

    if relay_url.is_empty() {
        return None;
    }

    Some(relay_url)
}

/// Registra as permissões de acesso ao sistema de arquivos no Tauri usando Early Returns (Guard Clauses)
/// para manter a complexidade ciclomática mínima e o código linear.
pub async fn setup_scopes_from_store<R: tauri::Runtime>(
    app_handle: &tauri::AppHandle<R>, app_data_directory: &std::path::Path,
) {
    let library_path = match read_library_path(app_data_directory) {
        Some(path) => path,
        None => {
            tracing::warn!("[Bios::Scopes] Key 'library_path' missing or invalid in settings.json");
            return;
        },
    };

    apply_library_scope(app_handle, &library_path);
}

fn apply_library_scope<R: tauri::Runtime>(
    app_handle: &tauri::AppHandle<R>, library_path: &std::path::Path,
) {
    tracing::info!("[Bios::Scopes] Registering filesystem scope for {:?}", library_path);

    if let Err(scope_error) = app_handle.fs_scope().allow_directory(library_path, true) {
        tracing::error!("[Bios::Scopes] Failed to allow directory in fs_scope: {}", scope_error);
    }

    if let Err(scope_error) = app_handle.asset_protocol_scope().allow_directory(library_path, true)
    {
        tracing::error!(
            "[Bios::Scopes] Failed to allow directory in asset_protocol_scope: {}",
            scope_error
        );
    }
}

#[cfg(test)]
mod tests {
    use tauri::Manager;
    use tauri_plugin_fs::FsExt;

    use super::{
        apply_library_scope, extract_library_path, read_library_path, read_relay_url_override,
    };

    #[test]
    fn test_extract_library_path_returns_the_configured_path() {
        let content = r#"{"library_path":"/home/user/comics"}"#;
        assert_eq!(
            extract_library_path(content),
            Some(std::path::PathBuf::from("/home/user/comics"))
        );
    }

    #[test]
    fn test_extract_library_path_missing_key_returns_none() {
        assert_eq!(extract_library_path(r#"{"other_key":"value"}"#), None);
    }

    #[test]
    fn test_read_library_path_returns_the_configured_path() {
        let app_data_directory = tempfile::tempdir().unwrap();
        std::fs::write(
            app_data_directory.path().join("settings.json"),
            r#"{"library_path":"/library/comics"}"#,
        )
        .unwrap();

        assert_eq!(
            read_library_path(app_data_directory.path()),
            Some(std::path::PathBuf::from("/library/comics"))
        );
    }

    #[test]
    fn test_read_relay_url_override_returns_the_configured_value() {
        let app_data_directory = tempfile::tempdir().unwrap();
        std::fs::write(
            app_data_directory.path().join("settings.json"),
            r#"{"relay_url":"https://relay.example.com"}"#,
        )
        .unwrap();

        assert_eq!(
            read_relay_url_override(app_data_directory.path()),
            Some("https://relay.example.com".to_string())
        );
    }

    #[test]
    fn test_read_relay_url_override_empty_value_returns_none() {
        let app_data_directory = tempfile::tempdir().unwrap();
        std::fs::write(app_data_directory.path().join("settings.json"), r#"{"relay_url":"  "}"#)
            .unwrap();

        assert_eq!(read_relay_url_override(app_data_directory.path()), None);
    }

    fn build_mock_app() -> tauri::App<tauri::test::MockRuntime> {
        tauri::test::mock_builder()
            .plugin(tauri_plugin_fs::init())
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap()
    }

    #[test]
    fn test_apply_library_scope_allows_the_directory() {
        let app = build_mock_app();
        let app_handle = app.handle();
        let library_directory = tempfile::tempdir().unwrap();

        apply_library_scope(app_handle, library_directory.path());

        assert!(app_handle.fs_scope().is_allowed(library_directory.path()));
        assert!(app_handle.asset_protocol_scope().is_allowed(library_directory.path()));
    }

    #[test]
    fn test_setup_scopes_from_store_registers_scope_from_settings() {
        let app = build_mock_app();
        let app_handle = app.handle();

        let app_data_directory = tempfile::tempdir().unwrap();
        let library_directory = tempfile::tempdir().unwrap();
        std::fs::write(
            app_data_directory.path().join("settings.json"),
            format!(r#"{{"library_path":{:?}}}"#, library_directory.path()),
        )
        .unwrap();

        tauri::async_runtime::block_on(super::setup_scopes_from_store(
            app_handle,
            app_data_directory.path(),
        ));

        assert!(app_handle.fs_scope().is_allowed(library_directory.path()));
        assert!(app_handle.asset_protocol_scope().is_allowed(library_directory.path()));
    }
}
