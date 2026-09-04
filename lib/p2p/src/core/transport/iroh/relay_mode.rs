//! Presets de configuração de relay expostos publicamente pela lib, resolvidos para
//! `iroh::RelayMode` no momento de montar o `Endpoint` (ver `IrohTransportBuilder::build_mode`).

use std::time::Duration;

use iroh::{RelayConfig, RelayMap, RelayMode as IrohRelayMode, RelayUrl, SecretKey};
use iroh_services::{
    caps::{create_api_token_from_secret_key, Caps, RelayCap},
    ApiSecret,
};

use crate::infra::error::ConnectionError;

/// URL do relay oficial mantido pelo ecossistema Acerola ("relay próprio", self-hosted,
/// aberto ao público — sem autenticação, diferente do relay da rede pública do Iroh abaixo).
pub const ACEROLA_DEFAULT_RELAY_URL: &str = "https://relay.acerola-comic.com";

/// Secret do projeto Acerola na Iroh Services (dashboard `services.iroh.computer`), embutido em
/// tempo de compilação via variável de ambiente do CI — não existe shell de usuário final pra
/// ler isso em runtime, e o app não pode depender de um `.env` no dispositivo de cada usuário.
/// Nunca é enviado pro relay: só serve pra assinar localmente um token de curta duração
/// vinculado à identidade deste node (ver [`RelayModeConfig::iroh_default_mode`]). Ausente em
/// builds locais/dev sem o secret configurado — nesse caso cai em `RelayMode::Default` sem
/// token, que o relay da n0 rejeita hoje (ver `IROH_SERVICES_API_SECRET` na doc do iroh).
const IROH_SERVICES_API_SECRET: Option<&str> = option_env!("IROH_SERVICES_API_SECRET");

/// Validade do token de autenticação derivado do secret do projeto — mesmo padrão (30 dias)
/// usado pelo preset oficial da `iroh_services` (`PresetBuilder::build`).
const RELAY_AUTH_TOKEN_TTL: Duration = Duration::from_secs(60 * 60 * 24 * 30);

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
    ///
    /// `node_secret` é a identidade (já resolvida/persistida) deste node — só é usada por
    /// [`RelayModeConfig::IrohDefault`], pra assinar um token de relay vinculado à chave pública
    /// deste node especificamente (ver [`Self::iroh_default_mode`]).
    pub(crate) fn resolve(
        &self, node_secret: &SecretKey,
    ) -> Result<IrohRelayMode, ConnectionError> {
        match self {
            RelayModeConfig::MdnsOnly => Ok(IrohRelayMode::Disabled),
            RelayModeConfig::IrohDefault => {
                Self::iroh_default_mode(node_secret, IROH_SERVICES_API_SECRET)
            },
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

    /// A rede pública/oficial do Iroh (n0) hoje exige autenticação — relays "Shared"/"Dedicated"
    /// da Iroh Services só aceitam tráfego de endpoints portando um token emitido a partir do
    /// secret do projeto (ver `IROH_SERVICES_API_SECRET` na doc do iroh, "authenticated
    /// relays"). Sem esse token, a conexão com o relay é recusada.
    ///
    /// O secret nunca é enviado pro relay: só assina localmente um token (rcan) vinculado à
    /// chave pública de `node_secret`, com validade de 30 dias, escopado só a uso de relay
    /// (`Caps::relay_use`) — mesmo mecanismo do preset oficial `iroh_services::preset()`, só que
    /// aplicado sobre a identidade que este builder já resolveu, em vez de gerar uma nova.
    ///
    /// `api_secret_ticket` recebido por parâmetro (em vez de ler `IROH_SERVICES_API_SECRET`
    /// direto aqui) só pra manter a função testável — `resolve()` é quem de fato lê a constante
    /// embutida em tempo de compilação.
    fn iroh_default_mode(
        node_secret: &SecretKey, api_secret_ticket: Option<&str>,
    ) -> Result<IrohRelayMode, ConnectionError> {
        let Some(embedded_secret) = api_secret_ticket else {
            tracing::warn!(
                "[RelayMode] IROH_SERVICES_API_SECRET não embutido neste build — a rede \
                 pública do Iroh vai recusar a conexão sem um token de autenticação válido."
            );
            return Ok(IrohRelayMode::Default);
        };

        let api_secret: ApiSecret = embedded_secret.parse().map_err(|err| {
            ConnectionError::StartupFailed(format!("invalid IROH_SERVICES_API_SECRET: {err}"))
        })?;

        let token = create_api_token_from_secret_key(
            api_secret.secret,
            node_secret.public(),
            RELAY_AUTH_TOKEN_TTL,
            Caps::new([RelayCap::Use]),
        )
        .map_err(|err| {
            ConnectionError::StartupFailed(format!("failed to create relay auth token: {err}"))
        })?;

        let mut encoded_token = data_encoding::BASE32_NOPAD.encode(&token.encode());
        encoded_token.make_ascii_lowercase();

        let relay_map =
            iroh::endpoint::default_relay_mode().relay_map().with_auth_token(encoded_token);

        Ok(IrohRelayMode::Custom(relay_map))
    }
}

#[cfg(test)]
mod tests {
    use iroh_services::ApiSecret;

    use super::*;

    fn node_secret() -> SecretKey {
        SecretKey::generate()
    }

    /// Ticket de API secret válido (formato real, encode/decode via `iroh_services::ApiSecret`),
    /// mas com uma chave gerada localmente — nunca um secret de produção de verdade.
    fn fake_api_secret_ticket() -> String {
        let secret = node_secret();
        let remote_id = node_secret().public();
        ApiSecret::new(secret, remote_id).to_string()
    }

    #[test]
    fn resolve_mdns_only_returns_disabled() {
        assert_eq!(
            RelayModeConfig::MdnsOnly.resolve(&node_secret()).unwrap(),
            IrohRelayMode::Disabled
        );
    }

    #[test]
    fn resolve_acerola_own_returns_custom_map_with_default_url() {
        let resolved = RelayModeConfig::AcerolaOwn.resolve(&node_secret()).unwrap();
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
        .resolve(&node_secret())
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
        let result =
            RelayModeConfig::Custom(vec!["not-a-valid-url".to_string()]).resolve(&node_secret());
        assert!(matches!(result, Err(ConnectionError::StartupFailed(_))));
    }

    #[test]
    fn resolve_custom_empty_list_returns_empty_custom_map() {
        let resolved = RelayModeConfig::Custom(vec![]).resolve(&node_secret()).unwrap();
        let IrohRelayMode::Custom(map) = resolved else {
            panic!("expected RelayMode::Custom, got {resolved:?}");
        };
        assert!(map.is_empty());
    }

    #[test]
    fn iroh_default_without_embedded_secret_falls_back_to_unauthenticated_default() {
        let resolved = RelayModeConfig::iroh_default_mode(&node_secret(), None).unwrap();
        assert_eq!(resolved, IrohRelayMode::Default);
    }

    #[test]
    fn iroh_default_with_invalid_secret_ticket_returns_startup_failed_error() {
        let result = RelayModeConfig::iroh_default_mode(&node_secret(), Some("not-a-valid-ticket"));
        assert!(matches!(result, Err(ConnectionError::StartupFailed(_))));
    }

    #[test]
    fn iroh_default_with_valid_secret_returns_authenticated_custom_map() {
        let ticket = fake_api_secret_ticket();
        let resolved = RelayModeConfig::iroh_default_mode(&node_secret(), Some(&ticket)).unwrap();

        let IrohRelayMode::Custom(map) = resolved else {
            panic!("expected RelayMode::Custom, got {resolved:?}");
        };

        // Mesmas URLs do relay map público padrão do iroh (n0), só que autenticadas.
        let default_map = iroh::endpoint::default_relay_mode().relay_map();
        assert_eq!(map.len(), default_map.len());
        assert!(!map.is_empty());

        for relay in map.relays::<Vec<_>>() {
            assert!(relay.auth_token.is_some(), "expected every relay to carry an auth token");
        }
    }

    #[test]
    fn iroh_default_token_differs_per_node_identity() {
        let ticket = fake_api_secret_ticket();

        let resolved_a = RelayModeConfig::iroh_default_mode(&node_secret(), Some(&ticket)).unwrap();
        let resolved_b = RelayModeConfig::iroh_default_mode(&node_secret(), Some(&ticket)).unwrap();

        let (IrohRelayMode::Custom(map_a), IrohRelayMode::Custom(map_b)) = (resolved_a, resolved_b)
        else {
            panic!("expected both resolutions to be RelayMode::Custom");
        };

        let token_a = map_a.relays::<Vec<_>>()[0].auth_token.clone();
        let token_b = map_b.relays::<Vec<_>>()[0].auth_token.clone();
        assert_ne!(token_a, token_b, "token deve ser vinculado à identidade do node, não fixo");
    }
}
