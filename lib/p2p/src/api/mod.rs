//! Fachada pública e Builder do nó Acerola.
//!
//! Este módulo exporta ordenadamente todos os primitivos consumidos pelas aplicações finais
//! de modo a ocultar o path complexo das pastas internas. Ele centraliza
//! o padrão de construção (Builder pattern) usado para inicializar uma instância P2p completa.

/// Renomeação transparente dos tipos de exceções manipulados no ecossistema P2p.
pub mod error {
    pub use crate::infra::error::ConnectionError as P2pError;
}
/// Abstração de armazenamento e transferência de blobs content-addressed.
pub mod blobs {
    #[cfg(feature = "iroh-blobs-adapter")]
    pub use crate::core::blobs::iroh::{IrohBlobStore, IrohBlobsConfig};
    pub use crate::core::blobs::{BlobHash, BlobHashParseError, InMemoryBlobStore, P2pBlobStore};
}

/// Utilitários ligados ao sistema de middlewares (Guards) de rede.
pub mod guard {
    pub use crate::core::guard::{
        open::OpenGuard,
        tofu::{InMemoryTrustedStore, TofuGuard, TrustedPeerStore},
        BoxedValidator as Guard, ConnectionContext,
    };
}

/// Encapsulamento da identificação de instâncias ligadas ao P2p.
pub mod peer {
    pub use crate::infra::peer::{PeerAddr, PeerId as PeerIdentity};
}

/// Interfaces essenciais e contratos que descrevem lógicas customizadas.
pub mod protocol {
    pub use crate::data::protocol::{EventEmitter, ProtocolHandler as Handler};
}

/// Enums descritivos de tipologias do protocolo.
pub mod network {
    pub use crate::core::network::state::NetworkMode;
}

/// Builder de transports
pub mod transport {
    pub use crate::core::transport::{
        iroh::{IrohTransportBuilder, RelayModeConfig, ACEROLA_DEFAULT_RELAY_URL},
        IncomingConnection, P2pTransport, TransportP2pBuilder,
    };
}

/// Contratos e implementações de persistência de dados.
pub mod storage {
    pub use crate::core::storage::{InMemoryStorage, P2PStorage};
}

/// Entidades dentro de um p2p
pub mod identity {
    #[cfg(target_os = "android")]
    pub use crate::core::device::android::DefaultDeviceInfoProvider;
    #[cfg(target_os = "linux")]
    pub use crate::core::device::linux::DefaultDeviceInfoProvider;
    #[cfg(target_os = "windows")]
    pub use crate::core::device::windows::DefaultDeviceInfoProvider;
    pub use crate::data::identity::{
        device_info::{DeviceInfo, DeviceInfoProvider},
        generate_seed,
    };
}

mod acerola_builder;
mod acerola_p2p;

pub use acerola_builder::AcerolaP2pBuilder;
pub use acerola_p2p::AcerolaP2p;
