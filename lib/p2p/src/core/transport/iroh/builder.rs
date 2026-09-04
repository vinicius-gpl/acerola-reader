use async_trait::async_trait;
use iroh::{
    endpoint::{self, presets},
    Endpoint, RelayConfig, RelayMap, RelayUrl, SecretKey,
};
use iroh_mdns_address_lookup as mdns;
use secrecy::{ExposeSecret, SecretBox};

use super::{
    blobs_bridge::BlobsIntegration, relay_mode::RelayModeConfig, transport::IrohTransport,
};
use crate::{core::transport::TransportP2pBuilder, infra::error::ConnectionError};

const IDENTITY_DERIVE_CONTEXT: &str = "acerola-p2p 2026 node identity";

/// Construtor configurável para o `IrohTransport`.
#[derive(Default)]
pub struct IrohTransportBuilder {
    relay_urls: Vec<String>,
    relay_mode: Option<RelayModeConfig>,
    seed: Option<SecretBox<[u8; 32]>>,
    blobs_config: super::blobs_bridge::BlobsConfigSlot,
}

impl IrohTransportBuilder {
    /// Adiciona uma URL de relay ao pool gerenciado pelo Iroh.
    ///
    /// Atalho equivalente a `.relay_mode(RelayModeConfig::Custom(vec![url]))` — mantido por
    /// compatibilidade com código existente. Ignorado se `.relay_mode(...)` também for chamado.
    pub fn relay(mut self, url: &str) -> Self {
        self.relay_urls.push(url.to_string());
        self
    }

    /// Define o modo de relay via um dos 4 presets suportados pela lib: sem relay remoto
    /// (`MdnsOnly`), relay(s) customizado(s) informado(s) pelo app (`Custom`), o relay oficial
    /// do Acerola (`AcerolaOwn`) ou a rede pública padrão do Iroh (`IrohDefault`).
    ///
    /// Tem prioridade sobre `.relay(url)` — se ambos forem chamados no mesmo builder, este
    /// vence e as URLs acumuladas via `.relay()` são ignoradas.
    pub fn relay_mode(mut self, mode: RelayModeConfig) -> Self {
        self.relay_mode = Some(mode);
        self
    }

    /// Define a seed bruta de 32 bytes para a geração de identidade criptográfica do nó.
    pub fn seed(mut self, seed: [u8; 32]) -> Self {
        self.seed = Some(SecretBox::new(Box::new(seed)));
        self
    }

    /// Ativa o adapter de blobs (`iroh-blobs`) sobre este transporte, usando a configuração de
    /// storage local informada.
    #[cfg(feature = "iroh-blobs-adapter")]
    pub fn blobs(mut self, config: crate::core::blobs::iroh::IrohBlobsConfig) -> Self {
        self.blobs_config = Some(config);
        self
    }
}

#[async_trait]
impl TransportP2pBuilder for IrohTransportBuilder {
    type Output = IrohTransport;

    fn set_seed(&mut self, seed: [u8; 32]) {
        self.seed = Some(SecretBox::new(Box::new(seed)));
    }

    fn get_seed(&self) -> Option<[u8; 32]> {
        self.seed.as_ref().map(|s| *s.expose_secret())
    }

    async fn build(self, alpns: Vec<Vec<u8>>) -> Result<IrohTransport, ConnectionError> {
        let mut alpns = alpns;
        if let Some(alpn) = super::blobs_bridge::wants_alpn(&self.blobs_config) {
            alpns.push(alpn.to_vec());
        }

        tracing::debug!(
            layer = "iroh_transport",
            alpns = ?alpns.iter().map(|it| String::from_utf8_lossy(it)).collect::<Vec<_>>(),
            "building iroh transport"
        );

        let mut builder = self.build_mode(alpns)?;
        builder = self.apply_secret(builder);

        let endpoint = builder.bind().await?;

        tracing::info!(
            layer = "iroh_transport",
            local_id = %endpoint.id(),
            "iroh transport bound successfully"
        );

        let blobs = BlobsIntegration::configure(&self.blobs_config, &endpoint).await?;
        Ok(IrohTransport::new(endpoint, blobs))
    }
}

#[rustfmt::skip]
impl IrohTransportBuilder {
    fn build_mode(&self, alpns: Vec<Vec<u8>>) -> Result<endpoint::Builder, ConnectionError> {
        let mdns = mdns::MdnsAddressLookup::builder();
        let builder = Endpoint::builder(presets::N0).address_lookup(mdns).alpns(alpns);

        let iroh_relay_mode = match &self.relay_mode {
            // `RelayModeConfig::IrohDefault` precisa saber a chave pública do node ANTES do
            // `.bind()` (ver `relay_mode.rs::iroh_default_mode`) — o token de autenticação da
            // rede pública do Iroh é vinculado a essa identidade, não emitido em branco.
            Some(config) => config.resolve(&self.derive_node_secret())?,
            None if self.relay_urls.is_empty() => iroh::RelayMode::Disabled,
            None => {
                let relay_configs: Vec<RelayConfig> = self.relay_urls.iter()
                    .map(|url| url.parse::<RelayUrl>().map(
                        RelayConfig::from
                    )).collect::<Result<_, iroh::RelayUrlParseError>>()?;

                iroh::RelayMode::Custom(RelayMap::from_iter(relay_configs))
            }
        };

        Ok(builder.relay_mode(iroh_relay_mode))
    }

    fn apply_secret(&self, mut builder: endpoint::Builder) -> endpoint::Builder {
        if self.seed.is_some() {
            builder = builder.secret_key(self.derive_node_secret());
        }

        builder
    }

    /// Deriva a `SecretKey` deste node a partir da seed já resolvida (ver
    /// `AcerolaP2pBuilder::resolve_identity`, que sempre define uma seed antes de `.build()`
    /// rodar em produção). Sem seed (só alcançável construindo este builder isoladamente, fora
    /// do fluxo normal — ver testes), gera uma chave efêmera só pra não travar a resolução do
    /// modo de relay; `apply_secret` continua deixando o `.bind()` gerar a identidade real nesse
    /// caso, então essa chave efêmera nunca é a que o node de fato assume.
    fn derive_node_secret(&self) -> SecretKey {
        match self.seed.as_ref() {
            Some(secret) => {
                let derive = blake3::derive_key(IDENTITY_DERIVE_CONTEXT, secret.expose_secret());
                SecretKey::from_bytes(&derive)
            },
            None => SecretKey::generate(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn validate_build_returns_transport() {
        let transport = IrohTransportBuilder::default()
            .relay("https://relay.test.local")
            .build(vec![b"test/proto".to_vec()]);
        assert!(transport.await.is_ok());
    }

    #[tokio::test]
    async fn validate_build_without_relay_returns_transport() {
        let transport = IrohTransportBuilder::default().build(vec![b"test/proto".to_vec()]);
        assert!(transport.await.is_ok());
    }

    #[tokio::test]
    async fn validate_build_invalid_relay_returns_error() {
        let transport = IrohTransportBuilder::default()
            .relay("not-a-valid-url")
            .build(vec![b"test/proto".to_vec()]);
        assert!(transport.await.is_err());
    }

    #[test]
    fn get_seed_returns_none_when_unconfigured_and_some_when_set() {
        let builder_unconfigured = IrohTransportBuilder::default();
        assert!(builder_unconfigured.get_seed().is_none());

        let target_seed = [0x55u8; 32];
        let mut builder_configured = IrohTransportBuilder::default();
        builder_configured.set_seed(target_seed);
        assert_eq!(builder_configured.get_seed(), Some(target_seed));
    }

    #[tokio::test]
    async fn relay_mode_mdns_only_builds_successfully() {
        let transport = IrohTransportBuilder::default()
            .relay_mode(RelayModeConfig::MdnsOnly)
            .build(vec![b"test/proto".to_vec()]);
        assert!(transport.await.is_ok());
    }

    #[tokio::test]
    async fn relay_mode_iroh_default_builds_successfully() {
        let transport = IrohTransportBuilder::default()
            .relay_mode(RelayModeConfig::IrohDefault)
            .build(vec![b"test/proto".to_vec()]);
        assert!(transport.await.is_ok());
    }

    #[tokio::test]
    async fn relay_mode_acerola_own_builds_successfully() {
        let transport = IrohTransportBuilder::default()
            .relay_mode(RelayModeConfig::AcerolaOwn)
            .build(vec![b"test/proto".to_vec()]);
        assert!(transport.await.is_ok());
    }

    #[tokio::test]
    async fn relay_mode_custom_valid_url_builds_successfully() {
        let transport = IrohTransportBuilder::default()
            .relay_mode(RelayModeConfig::Custom(vec!["https://relay.test.local".to_string()]))
            .build(vec![b"test/proto".to_vec()]);
        assert!(transport.await.is_ok());
    }

    #[tokio::test]
    async fn relay_mode_custom_invalid_url_returns_error() {
        let transport = IrohTransportBuilder::default()
            .relay_mode(RelayModeConfig::Custom(vec!["not-a-valid-url".to_string()]))
            .build(vec![b"test/proto".to_vec()]);
        assert!(transport.await.is_err());
    }

    #[tokio::test]
    async fn relay_mode_takes_priority_over_legacy_relay_urls() {
        // A URL legada é claramente inválida — se `.relay_mode(...)` não tivesse prioridade,
        // o build falharia ao tentar resolvê-la.
        let transport = IrohTransportBuilder::default()
            .relay("not-a-valid-url")
            .relay_mode(RelayModeConfig::MdnsOnly)
            .build(vec![b"test/proto".to_vec()]);
        assert!(transport.await.is_ok());
    }
}
