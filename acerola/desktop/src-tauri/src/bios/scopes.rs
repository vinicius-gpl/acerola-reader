use std::path::PathBuf;

use acerola_p2p::api::transport::{RelayModeConfig, ACEROLA_DEFAULT_RELAY_URL};
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

/// Configuração de relay resolvida a partir de `settings.json`, combinável em múltiplas
/// fontes simultâneas (ver [`RelaySettings::resolve`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelaySettings {
    /// Usa o relay oficial mantido pelo Acerola (`ACEROLA_DEFAULT_RELAY_URL`).
    pub use_acerola_relay: bool,
    /// Usa a rede pública padrão de relays do projeto Iroh (n0) — mutuamente exclusiva com
    /// as demais fontes: `iroh::RelayMode` só permite `Disabled | Default | Custom`, nunca
    /// uma combinação de `Default` com URLs específicas.
    pub use_iroh_public_network: bool,
    /// Relay(s) próprio(s) do usuário (self-hosted, sem autenticação).
    pub custom_relay_urls: Vec<String>,
}

impl Default for RelaySettings {
    fn default() -> Self {
        Self {
            use_acerola_relay: true,
            use_iroh_public_network: false,
            custom_relay_urls: Vec::new(),
        }
    }
}

impl RelaySettings {
    /// Resolve as fontes habilitadas para o `RelayModeConfig` concreto consumido pelo
    /// `IrohTransportBuilder`. Sem nenhuma fonte ativa, cai em `MdnsOnly` — não existe um
    /// modo "mDNS only" explícito na UI, é só o estado natural de "nada selecionado".
    ///
    /// `iroh_services_ticket` vem do cofre criptografado (`SecureP2pStorage`), não de
    /// `settings.json` — é a conta do PRÓPRIO usuário em `services.iroh.computer`, nunca um
    /// secret de projeto embutido no build. Se o toggle estiver ligado mas nenhum ticket
    /// tiver sido colado ainda, essa fonte é ignorada (não há como montar
    /// `RelayModeConfig::IrohDefault` sem ele) e as demais fontes combinam normalmente.
    pub fn resolve(&self, iroh_services_ticket: Option<&str>) -> RelayModeConfig {
        if self.use_iroh_public_network {
            if let Some(ticket) = iroh_services_ticket {
                return RelayModeConfig::IrohDefault(ticket.to_string());
            }
        }

        let mut urls = Vec::new();
        if self.use_acerola_relay {
            urls.push(ACEROLA_DEFAULT_RELAY_URL.to_string());
        }
        urls.extend(self.custom_relay_urls.iter().cloned());

        if urls.is_empty() {
            RelayModeConfig::MdnsOnly
        } else {
            RelayModeConfig::Custom(urls)
        }
    }
}

/// Lê a configuração de relay combinável de `settings.json` (`relay_use_acerola`,
/// `relay_use_iroh_public`, `relay_custom_urls`). Ausência total do arquivo/chaves cai no
/// [`RelaySettings::default`] (relay do Acerola habilitado, sem mais nenhuma fonte). Só é lida
/// na inicialização: trocar a configuração de relay em runtime não é suportado pela lib, só a
/// troca de modo local/relay (já exposta via `switch_to_local`/`switch_to_relay`).
///
/// Migração: se nenhuma das chaves novas estiver presente mas a chave legada `relay_url` (de
/// antes desta feature, URL única) estiver, ela é migrada para dentro de `custom_relay_urls` —
/// sem isso, quem já tinha configurado um relay próprio perderia essa escolha silenciosamente
/// no primeiro boot após o update. `relay_iroh_urls` (lista separada "Relays Iroh", removida por
/// ser funcionalmente idêntica a `relay_custom_urls` — as duas viravam o mesmo
/// `RelayModeConfig::Custom`) é sempre mesclada em `custom_relay_urls` se presente, mesmo
/// quando as chaves novas já existem.
pub fn read_relay_settings(app_data_directory: &std::path::Path) -> RelaySettings {
    let settings_file_path = app_data_directory.join("settings.json");
    let Ok(file_content) = std::fs::read_to_string(&settings_file_path) else {
        return RelaySettings::default();
    };
    let Ok(json_value) = serde_json::from_str::<Value>(&file_content) else {
        return RelaySettings::default();
    };

    let legacy_iroh_urls = read_string_array(&json_value, "relay_iroh_urls");

    if !has_any_relay_settings_key(&json_value) {
        if let Some(legacy_url) = read_legacy_relay_url(&json_value) {
            tracing::info!(
                "[Bios::Scopes] Migrating legacy 'relay_url' override into 'relay_custom_urls'"
            );
            let mut custom_relay_urls = vec![legacy_url];
            custom_relay_urls.extend(legacy_iroh_urls);
            return RelaySettings { custom_relay_urls, ..RelaySettings::default() };
        }
        if legacy_iroh_urls.is_empty() {
            return RelaySettings::default();
        }
        return RelaySettings { custom_relay_urls: legacy_iroh_urls, ..RelaySettings::default() };
    }

    let mut custom_relay_urls = read_string_array(&json_value, "relay_custom_urls");
    for url in legacy_iroh_urls {
        if !custom_relay_urls.contains(&url) {
            custom_relay_urls.push(url);
        }
    }

    RelaySettings {
        use_acerola_relay: json_value
            .get("relay_use_acerola")
            .and_then(Value::as_bool)
            .unwrap_or(true),
        use_iroh_public_network: json_value
            .get("relay_use_iroh_public")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        custom_relay_urls,
    }
}

fn has_any_relay_settings_key(json_value: &Value) -> bool {
    ["relay_use_acerola", "relay_use_iroh_public", "relay_custom_urls", "relay_iroh_urls"]
        .iter()
        .any(|key| json_value.get(key).is_some())
}

fn read_legacy_relay_url(json_value: &Value) -> Option<String> {
    let relay_url = json_value.get("relay_url")?.as_str()?.trim().to_string();
    if relay_url.is_empty() {
        return None;
    }
    Some(relay_url)
}

fn read_string_array(json_value: &Value, key: &str) -> Vec<String> {
    json_value
        .get(key)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|url| !url.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// Lê o apelido custom do dispositivo local (`device_alias`, estilo LocalSend) de
/// `settings.json`. Ausente/vazio (caso comum) faz o chamador cair no hostname automático do
/// `DeviceInfoProvider`. É lido só na inicialização do node P2P — depois disso, um apelido
/// definido em runtime (`set_local_device_name`) já não passa mais por aqui, só é persistido
/// nesta chave pelo frontend pra sobreviver ao próximo restart.
pub fn read_device_alias_override(app_data_directory: &std::path::Path) -> Option<String> {
    let settings_file_path = app_data_directory.join("settings.json");
    let file_content = std::fs::read_to_string(&settings_file_path).ok()?;
    let json_value: Value = serde_json::from_str(&file_content).ok()?;
    let alias = json_value.get("device_alias")?.as_str()?.trim().to_string();

    if alias.is_empty() {
        return None;
    }

    Some(alias)
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
    use acerola_p2p::api::transport::RelayModeConfig;
    use tauri::Manager;
    use tauri_plugin_fs::FsExt;

    use super::{
        apply_library_scope, extract_library_path, read_device_alias_override, read_library_path,
        read_relay_settings, RelaySettings,
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
    fn test_read_relay_settings_missing_file_returns_default() {
        let app_data_directory = tempfile::tempdir().unwrap();
        assert_eq!(read_relay_settings(app_data_directory.path()), RelaySettings::default());
    }

    #[test]
    fn test_read_relay_settings_reads_all_configured_fields() {
        let app_data_directory = tempfile::tempdir().unwrap();
        std::fs::write(
            app_data_directory.path().join("settings.json"),
            r#"{
                "relay_use_acerola": false,
                "relay_use_iroh_public": false,
                "relay_custom_urls": ["https://relay-a.test.local", "  ", "https://relay-b.test.local"]
            }"#,
        )
        .unwrap();

        assert_eq!(
            read_relay_settings(app_data_directory.path()),
            RelaySettings {
                use_acerola_relay: false,
                use_iroh_public_network: false,
                custom_relay_urls: vec![
                    "https://relay-a.test.local".to_string(),
                    "https://relay-b.test.local".to_string()
                ],
            }
        );
    }

    /// `relay_iroh_urls` era uma lista separada ("Relays Iroh") funcionalmente idêntica a
    /// `relay_custom_urls` (as duas viravam o mesmo `RelayModeConfig::Custom`) — removida da
    /// UI, mas quem já tinha algo salvo ali não pode perder essa URL silenciosamente.
    #[test]
    fn test_read_relay_settings_merges_legacy_iroh_urls_into_custom_urls() {
        let app_data_directory = tempfile::tempdir().unwrap();
        std::fs::write(
            app_data_directory.path().join("settings.json"),
            r#"{
                "relay_custom_urls": ["https://relay-a.test.local"],
                "relay_iroh_urls": ["https://iroh-relay.test.local"]
            }"#,
        )
        .unwrap();

        assert_eq!(
            read_relay_settings(app_data_directory.path()),
            RelaySettings {
                custom_relay_urls: vec![
                    "https://relay-a.test.local".to_string(),
                    "https://iroh-relay.test.local".to_string()
                ],
                ..RelaySettings::default()
            }
        );
    }

    #[test]
    fn test_read_relay_settings_migrates_legacy_iroh_urls_only_when_no_other_key_present() {
        let app_data_directory = tempfile::tempdir().unwrap();
        std::fs::write(
            app_data_directory.path().join("settings.json"),
            r#"{"relay_iroh_urls": ["https://iroh-relay.test.local"]}"#,
        )
        .unwrap();

        assert_eq!(
            read_relay_settings(app_data_directory.path()),
            RelaySettings {
                custom_relay_urls: vec!["https://iroh-relay.test.local".to_string()],
                ..RelaySettings::default()
            }
        );
    }

    #[test]
    fn test_read_relay_settings_does_not_duplicate_url_present_in_both_lists() {
        let app_data_directory = tempfile::tempdir().unwrap();
        std::fs::write(
            app_data_directory.path().join("settings.json"),
            r#"{
                "relay_custom_urls": ["https://relay-a.test.local"],
                "relay_iroh_urls": ["https://relay-a.test.local"]
            }"#,
        )
        .unwrap();

        assert_eq!(
            read_relay_settings(app_data_directory.path()),
            RelaySettings {
                custom_relay_urls: vec!["https://relay-a.test.local".to_string()],
                ..RelaySettings::default()
            }
        );
    }

    #[test]
    fn test_read_relay_settings_migrates_legacy_relay_url_into_custom_urls() {
        let app_data_directory = tempfile::tempdir().unwrap();
        std::fs::write(
            app_data_directory.path().join("settings.json"),
            r#"{"relay_url":"https://relay.example.com"}"#,
        )
        .unwrap();

        assert_eq!(
            read_relay_settings(app_data_directory.path()),
            RelaySettings {
                custom_relay_urls: vec!["https://relay.example.com".to_string()],
                ..RelaySettings::default()
            }
        );
    }

    #[test]
    fn test_read_relay_settings_empty_legacy_relay_url_returns_default() {
        let app_data_directory = tempfile::tempdir().unwrap();
        std::fs::write(app_data_directory.path().join("settings.json"), r#"{"relay_url":"  "}"#)
            .unwrap();

        assert_eq!(read_relay_settings(app_data_directory.path()), RelaySettings::default());
    }

    #[test]
    fn test_relay_settings_resolve_nothing_active_returns_mdns_only() {
        let settings = RelaySettings {
            use_acerola_relay: false,
            use_iroh_public_network: false,
            custom_relay_urls: vec![],
        };
        assert_eq!(settings.resolve(None), RelayModeConfig::MdnsOnly);
    }

    #[test]
    fn test_relay_settings_resolve_merges_all_active_sources_into_custom() {
        let settings = RelaySettings {
            use_acerola_relay: true,
            use_iroh_public_network: false,
            custom_relay_urls: vec!["https://relay-a.test.local".to_string()],
        };

        assert_eq!(
            settings.resolve(None),
            RelayModeConfig::Custom(vec![
                acerola_p2p::api::transport::ACEROLA_DEFAULT_RELAY_URL.to_string(),
                "https://relay-a.test.local".to_string(),
            ])
        );
    }

    #[test]
    fn test_relay_settings_resolve_iroh_public_network_with_ticket_ignores_other_sources() {
        let settings = RelaySettings {
            use_acerola_relay: true,
            use_iroh_public_network: true,
            custom_relay_urls: vec!["https://relay-a.test.local".to_string()],
        };

        assert_eq!(
            settings.resolve(Some("services-fake-ticket")),
            RelayModeConfig::IrohDefault("services-fake-ticket".to_string())
        );
    }

    /// Toggle ligado mas sem ticket colado ainda (primeiro uso, ou usuário desmarcou depois de
    /// já ter salvo em `settings.json`) — não há como montar `IrohDefault` sem o ticket, então
    /// cai pras demais fontes combinadas normalmente, como se o toggle estivesse desligado.
    #[test]
    fn test_relay_settings_resolve_iroh_public_network_without_ticket_falls_back_to_other_sources()
    {
        let settings = RelaySettings {
            use_acerola_relay: true,
            use_iroh_public_network: true,
            custom_relay_urls: vec![],
        };

        assert_eq!(
            settings.resolve(None),
            RelayModeConfig::Custom(vec![
                acerola_p2p::api::transport::ACEROLA_DEFAULT_RELAY_URL.to_string()
            ])
        );
    }

    #[test]
    fn test_read_device_alias_override_returns_the_configured_value() {
        let app_data_directory = tempfile::tempdir().unwrap();
        std::fs::write(
            app_data_directory.path().join("settings.json"),
            r#"{"device_alias":"Notebook do Vinicius"}"#,
        )
        .unwrap();

        assert_eq!(
            read_device_alias_override(app_data_directory.path()),
            Some("Notebook do Vinicius".to_string())
        );
    }

    #[test]
    fn test_read_device_alias_override_missing_key_returns_none() {
        let app_data_directory = tempfile::tempdir().unwrap();
        std::fs::write(app_data_directory.path().join("settings.json"), r#"{}"#).unwrap();

        assert_eq!(read_device_alias_override(app_data_directory.path()), None);
    }

    #[test]
    fn test_read_device_alias_override_empty_value_returns_none() {
        let app_data_directory = tempfile::tempdir().unwrap();
        std::fs::write(
            app_data_directory.path().join("settings.json"),
            r#"{"device_alias":"   "}"#,
        )
        .unwrap();

        assert_eq!(read_device_alias_override(app_data_directory.path()), None);
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
