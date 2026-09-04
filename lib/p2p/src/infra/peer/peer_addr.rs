use crate::infra::peer::PeerId;

/// Endereço de conexão de um peer contendo o identificador do nó e os dados de endereçamento.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PeerAddr {
    /// Identificador do peer.
    pub id: PeerId,
    /// Bytes de endereçamento serializado (ex: EndpointAddr do Iroh).
    pub addrs: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]
        #[test]
        fn peer_addr_serialization_roundtrip_proptest(
            peer_identifier in ".*",
            device_identifier in proptest::option::of(".*"),
            address_bytes in proptest::collection::vec(proptest::num::u8::ANY, 0..512)
        ) {
            let original_address = PeerAddr {
                id: PeerId {
                    id: peer_identifier,
                    device_id: device_identifier,
                },
                addrs: address_bytes,
            };

            let json_representation =
                serde_json::to_string(&original_address).expect("Failed to serialize PeerAddr");
            let deserialized_address: PeerAddr =
                serde_json::from_str(&json_representation).expect("Failed to deserialize PeerAddr");

            prop_assert_eq!(original_address, deserialized_address);
        }
    }
}
