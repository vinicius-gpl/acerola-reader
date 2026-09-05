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
    /// Relay(s) próprio(s) do usuário (self-hosted, sem autenticação).
    pub custom_relay_urls: Vec<String>,
}

/// Erro exposto pra fronteira FFI (Kotlin) quando o usuário cola um ticket malformado —
/// validado antes de persistir, em vez de só falhar silenciosamente no próximo boot.
#[derive(uniffi::Error, thiserror::Error, Debug)]
pub enum RelayTicketError {
    #[error("invalid Iroh Services ticket: {reason}")]
    Invalid { reason: String },
}

impl FfiRelaySettings {
    /// Resolve as fontes habilitadas para o `RelayModeConfig` concreto consumido pelo
    /// `IrohTransportBuilder`. Sem nenhuma fonte ativa, cai em `MdnsOnly` — não existe um modo
    /// "mDNS only" explícito na UI, é só o estado natural de "nada selecionado".
    ///
    /// `iroh_services_ticket` vem do cofre criptografado (`SecureP2pStorage`), não do
    /// DataStore — é a conta do PRÓPRIO usuário em `services.iroh.computer`, uma credencial
    /// real, não uma preferência qualquer. Se o toggle estiver ligado mas nenhum ticket tiver
    /// sido colado ainda, essa fonte é ignorada e as demais combinam normalmente.
    pub(crate) fn resolve(&self, iroh_services_ticket: Option<&str>) -> RelayModeConfig {
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
        };

        assert_eq!(settings.resolve(None), RelayModeConfig::MdnsOnly);
    }

    #[test]
    fn test_resolve_merges_all_active_sources_into_custom() {
        let settings = FfiRelaySettings {
            use_acerola_relay: true,
            use_iroh_public_network: false,
            custom_relay_urls: vec!["https://relay-a.test.local".to_string()],
        };

        assert_eq!(
            settings.resolve(None),
            RelayModeConfig::Custom(vec![
                ACEROLA_DEFAULT_RELAY_URL.to_string(),
                "https://relay-a.test.local".to_string(),
            ])
        );
    }

    #[test]
    fn test_resolve_iroh_public_network_with_ticket_ignores_other_sources() {
        let settings = FfiRelaySettings {
            use_acerola_relay: true,
            use_iroh_public_network: true,
            custom_relay_urls: vec!["https://relay-a.test.local".to_string()],
        };

        assert_eq!(
            settings.resolve(Some("services-fake-ticket")),
            RelayModeConfig::IrohDefault("services-fake-ticket".to_string())
        );
    }

    #[test]
    fn test_resolve_iroh_public_network_without_ticket_falls_back_to_other_sources() {
        let settings = FfiRelaySettings {
            use_acerola_relay: true,
            use_iroh_public_network: true,
            custom_relay_urls: vec![],
        };

        assert_eq!(
            settings.resolve(None),
            RelayModeConfig::Custom(vec![ACEROLA_DEFAULT_RELAY_URL.to_string()])
        );
    }

    #[test]
    fn test_resolve_only_acerola_relay_active() {
        let settings = FfiRelaySettings {
            use_acerola_relay: true,
            use_iroh_public_network: false,
            custom_relay_urls: vec![],
        };

        assert_eq!(
            settings.resolve(None),
            RelayModeConfig::Custom(vec![ACEROLA_DEFAULT_RELAY_URL.to_string()])
        );
    }
}
