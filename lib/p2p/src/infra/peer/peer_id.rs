//! Definição da identidade de um nó (Peer) na rede P2P.
//!
//! Este módulo provê a estrutura `PeerId`, que encapsula o identificador
//! de um nó, sendo a principal forma de referenciar destinos.

use std::fmt;

use uuid::Uuid;

/// INFO: Chave para fazer chaves derivadas deterministicas.
const ACEROLA_DEVICE_NAMESPACE: Uuid = Uuid::from_bytes(*b"acerola-p2p!dev!");

/// Representa a identidade pública de um nó na rede.
///
/// O `PeerId` contém o identificador subjacente da camada de rede (ex: Iroh PublicKey
/// formatado). Implementa traits de equivalência e hashing para uso como chave
/// no rastreamento do estado das conexões ativas.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PeerId {
    /// Identificador único (geralmente uma string em base32).
    pub id: String,
    /// Identificador único determinístico derivado da chave pública do dispositivo.
    pub device_id: Option<String>,
}

impl PeerId {
    /// Cria um novo `PeerId` derivando o `device_id` deterministicamente a partir dos bytes da chave pública.
    pub fn from_public_key(id: String, public_key_bytes: &[u8]) -> Self {
        let device_id = Uuid::new_v5(&ACEROLA_DEVICE_NAMESPACE, public_key_bytes).to_string();
        Self { id, device_id: Some(device_id) }
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]
        #[test]
        fn from_public_key_is_always_deterministic_proptest(
            peer_identifier in ".*",
            public_key_bytes in proptest::collection::vec(proptest::num::u8::ANY, 0..512)
        ) {
            let first_peer = PeerId::from_public_key(peer_identifier.clone(), &public_key_bytes);
            let second_peer = PeerId::from_public_key(peer_identifier, &public_key_bytes);

            prop_assert_eq!(first_peer.id, second_peer.id);
            prop_assert_eq!(first_peer.device_id.clone(), second_peer.device_id);
            prop_assert!(first_peer.device_id.is_some());
        }

        #[test]
        fn peer_id_serialization_roundtrip_proptest(
            peer_identifier in ".*",
            device_identifier in proptest::option::of(".*")
        ) {
            let original_peer_id = PeerId {
                id: peer_identifier,
                device_id: device_identifier,
            };

            let json_representation =
                serde_json::to_string(&original_peer_id).expect("Failed to serialize PeerId");
            let deserialized_peer_id: PeerId =
                serde_json::from_str(&json_representation).expect("Failed to deserialize PeerId");

            prop_assert_eq!(original_peer_id, deserialized_peer_id);
        }
    }

    const FAKE_KEY: &[u8] = &[0xab; 32];
    const ANOTHER_KEY: &[u8] = &[0xcd; 32];

    #[test]
    fn from_public_key_populates_device_id() {
        let peer = PeerId::from_public_key("node-1".to_string(), FAKE_KEY);
        assert!(peer.device_id.is_some());
    }

    #[test]
    fn from_public_key_is_deterministic() {
        let a = PeerId::from_public_key("node-1".to_string(), FAKE_KEY);
        let b = PeerId::from_public_key("node-1".to_string(), FAKE_KEY);
        assert_eq!(a.device_id, b.device_id);
    }

    #[test]
    fn different_bytes_generate_different_device_ids() {
        let a = PeerId::from_public_key("node-1".to_string(), FAKE_KEY);
        let b = PeerId::from_public_key("node-1".to_string(), ANOTHER_KEY);
        assert_ne!(a.device_id, b.device_id);
    }

    #[test]
    fn peer_without_bytes_has_none_device_id() {
        let peer = PeerId { id: "node-remote".to_string(), device_id: None };
        assert!(peer.device_id.is_none());
    }

    #[test]
    fn device_id_is_valid_uuid_v5() {
        let peer = PeerId::from_public_key("node-1".to_string(), FAKE_KEY);
        let uuid = Uuid::parse_str(peer.device_id.unwrap().as_str());
        assert!(uuid.is_ok());
    }

    #[test]
    fn id_does_not_affect_device_id() {
        let a = PeerId::from_public_key("node-1".to_string(), FAKE_KEY);
        let b = PeerId::from_public_key("node-2".to_string(), FAKE_KEY);
        assert_eq!(a.device_id, b.device_id);
    }
}
