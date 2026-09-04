use acerola_p2p::api::transport::{RelayModeConfig, ACEROLA_DEFAULT_RELAY_URL};

/// Configuração de relay combinável exposta pra fronteira FFI (Kotlin) — espelha
/// `RelaySettings`/`RelaySettings::resolve` do Desktop (`bios/scopes.rs`). A diferença é só de
/// onde as fontes vêm: no Desktop é lido de `settings.json` pelo próprio Rust, no Android é lido
/// do DataStore pelo Kotlin (`RelayPreference`) e passado inteiro aqui na construção do
/// `P2PNode` — a lógica de combinar as fontes num `RelayModeConfig` é a mesma nos dois apps.
#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq)]
pub struct FfiRelaySettings {
    /// Usa o relay oficial mantido pelo Acerola (`ACEROLA_DEFAULT_RELAY_URL`).
    pub use_acerola_relay: bool,
    /// Usa a rede pública padrão de relays do projeto Iroh (n0) — mutuamente exclusiva com as
    /// demais fontes: `iroh::RelayMode` só permite `Disabled | Default | Custom`, nunca uma
    /// combinação de `Default` com URLs específicas.
    pub use_iroh_public_network: bool,
    /// Relay(s) próprio(s) do usuário (self-hosted).
    pub custom_relay_urls: Vec<String>,
    /// Relay(s) que falam o protocolo Iroh, mas não fazem parte da rede pública n0.
    pub iroh_relay_urls: Vec<String>,
}

impl FfiRelaySettings {
    /// Resolve as fontes habilitadas para o `RelayModeConfig` concreto consumido pelo
    /// `IrohTransportBuilder`. Sem nenhuma fonte ativa, cai em `MdnsOnly` — não existe um modo
    /// "mDNS only" explícito na UI, é só o estado natural de "nada selecionado".
    pub(crate) fn resolve(&self) -> RelayModeConfig {
        if self.use_iroh_public_network {
            return RelayModeConfig::IrohDefault;
        }

        let mut urls = Vec::new();
        if self.use_acerola_relay {
            urls.push(ACEROLA_DEFAULT_RELAY_URL.to_string());
        }
        urls.extend(self.custom_relay_urls.iter().cloned());
        urls.extend(self.iroh_relay_urls.iter().cloned());

        if urls.is_empty() {
            RelayModeConfig::MdnsOnly
        } else {
            RelayModeConfig::Custom(urls)
        }
    }
}

#[cfg(test)]
mod tests {
    use acerola_p2p::api::transport::{RelayModeConfig, ACEROLA_DEFAULT_RELAY_URL};

    use super::FfiRelaySettings;

    #[test]
    fn test_resolve_nothing_active_returns_mdns_only() {
        let settings = FfiRelaySettings {
            use_acerola_relay: false,
            use_iroh_public_network: false,
            custom_relay_urls: vec![],
            iroh_relay_urls: vec![],
        };

        assert_eq!(settings.resolve(), RelayModeConfig::MdnsOnly);
    }

    #[test]
    fn test_resolve_merges_all_active_sources_into_custom() {
        let settings = FfiRelaySettings {
            use_acerola_relay: true,
            use_iroh_public_network: false,
            custom_relay_urls: vec!["https://relay-a.test.local".to_string()],
            iroh_relay_urls: vec!["https://iroh-relay.test.local".to_string()],
        };

        assert_eq!(
            settings.resolve(),
            RelayModeConfig::Custom(vec![
                ACEROLA_DEFAULT_RELAY_URL.to_string(),
                "https://relay-a.test.local".to_string(),
                "https://iroh-relay.test.local".to_string(),
            ])
        );
    }

    #[test]
    fn test_resolve_iroh_public_network_ignores_other_sources() {
        let settings = FfiRelaySettings {
            use_acerola_relay: true,
            use_iroh_public_network: true,
            custom_relay_urls: vec!["https://relay-a.test.local".to_string()],
            iroh_relay_urls: vec![],
        };

        assert_eq!(settings.resolve(), RelayModeConfig::IrohDefault);
    }

    #[test]
    fn test_resolve_only_acerola_relay_active() {
        let settings = FfiRelaySettings {
            use_acerola_relay: true,
            use_iroh_public_network: false,
            custom_relay_urls: vec![],
            iroh_relay_urls: vec![],
        };

        assert_eq!(
            settings.resolve(),
            RelayModeConfig::Custom(vec![ACEROLA_DEFAULT_RELAY_URL.to_string()])
        );
    }
}
