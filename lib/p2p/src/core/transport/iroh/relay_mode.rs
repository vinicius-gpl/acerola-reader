//! Presets de configuração de relay expostos publicamente pela lib, resolvidos para
//! `iroh::RelayMode` no momento de montar o `Endpoint` (ver `IrohTransportBuilder::build_mode`).

use iroh::{RelayConfig, RelayMap, RelayMode as IrohRelayMode, RelayUrl};

use crate::infra::error::ConnectionError;

/// URL do relay oficial mantido pelo ecossistema Acerola ("relay próprio").
pub const ACEROLA_DEFAULT_RELAY_URL: &str = "https://relay.acerola-comic.com";

/// Os 4 modos de relay suportados pela lib para transposição de NAT via `IrohTransportBuilder`.
///
/// mDNS de descoberta na LAN funciona independentemente do modo escolhido aqui — mesmo em
/// `MdnsOnly` o nó continua descobrindo vizinhos na rede local, só não tem nenhuma via remota
/// (relay) para peers fora da LAN.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelayModeConfig {
    /// Sem relay remoto algum — só descoberta mDNS na rede local (offline/alta privacidade).
    MdnsOnly,
    /// Relay(s) customizado(s) informado(s) pelo app consumidor ("meu relay").
    Custom(Vec<String>),
    /// Relay oficial mantido pelo Acerola (`ACEROLA_DEFAULT_RELAY_URL`, "relay próprio").
    AcerolaOwn,
    /// Rede pública de relays padrão do projeto Iroh (produção n0).
    IrohDefault,
}

impl RelayModeConfig {
    /// Resolve este preset para o `iroh::RelayMode` concreto usado pelo `endpoint::Builder`.
    pub(crate) fn resolve(&self) -> Result<IrohRelayMode, ConnectionError> {
        match self {
            RelayModeConfig::MdnsOnly => Ok(IrohRelayMode::Disabled),
            RelayModeConfig::IrohDefault => Ok(IrohRelayMode::Default),
            RelayModeConfig::AcerolaOwn => {
                Self::custom_mode(&[ACEROLA_DEFAULT_RELAY_URL.to_string()])
            },
            RelayModeConfig::Custom(urls) => Self::custom_mode(urls),
        }
    }

    /// Faz o parse de uma lista de URLs de relay em um `RelayMode::Custom`, propagando um
    /// `ConnectionError::StartupFailed` descritivo em caso de URL malformada.
    fn custom_mode(urls: &[String]) -> Result<IrohRelayMode, ConnectionError> {
        let relay_configs: Vec<RelayConfig> = urls
            .iter()
            .map(|url| url.parse::<RelayUrl>().map(RelayConfig::from))
            .collect::<Result<_, iroh::RelayUrlParseError>>()
            .map_err(|err| ConnectionError::StartupFailed(format!("invalid relay URL: {err}")))?;

        Ok(IrohRelayMode::Custom(RelayMap::from_iter(relay_configs)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_mdns_only_returns_disabled() {
        assert_eq!(RelayModeConfig::MdnsOnly.resolve().unwrap(), IrohRelayMode::Disabled);
    }

    #[test]
    fn resolve_iroh_default_returns_default() {
        assert_eq!(RelayModeConfig::IrohDefault.resolve().unwrap(), IrohRelayMode::Default);
    }

    #[test]
    fn resolve_acerola_own_returns_custom_map_with_default_url() {
        let resolved = RelayModeConfig::AcerolaOwn.resolve().unwrap();
        let IrohRelayMode::Custom(map) = resolved else {
            panic!("expected RelayMode::Custom, got {resolved:?}");
        };

        let expected_url: RelayUrl = ACEROLA_DEFAULT_RELAY_URL.parse().unwrap();
        assert!(map.contains(&expected_url));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn resolve_custom_valid_urls_returns_custom_map_with_all_urls() {
        let resolved = RelayModeConfig::Custom(vec![
            "https://relay-a.test.local".to_string(),
            "https://relay-b.test.local".to_string(),
        ])
        .resolve()
        .unwrap();

        let IrohRelayMode::Custom(map) = resolved else {
            panic!("expected RelayMode::Custom, got {resolved:?}");
        };

        assert_eq!(map.len(), 2);
        assert!(map.contains(&"https://relay-a.test.local".parse().unwrap()));
        assert!(map.contains(&"https://relay-b.test.local".parse().unwrap()));
    }

    #[test]
    fn resolve_custom_invalid_url_returns_startup_failed_error() {
        let result = RelayModeConfig::Custom(vec!["not-a-valid-url".to_string()]).resolve();
        assert!(matches!(result, Err(ConnectionError::StartupFailed(_))));
    }

    #[test]
    fn resolve_custom_empty_list_returns_empty_custom_map() {
        let resolved = RelayModeConfig::Custom(vec![]).resolve().unwrap();
        let IrohRelayMode::Custom(map) = resolved else {
            panic!("expected RelayMode::Custom, got {resolved:?}");
        };
        assert!(map.is_empty());
    }
}
