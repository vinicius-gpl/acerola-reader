use acerola_p2p::api::network::NetworkMode;

#[derive(uniffi::Enum, Clone, Debug, PartialEq, Eq)]
pub enum FfiNetworkMode {
    Local,
    Relay,
}

impl From<NetworkMode> for FfiNetworkMode {
    fn from(value: NetworkMode) -> Self {
        match value {
            NetworkMode::Local => FfiNetworkMode::Local,
            NetworkMode::Relay => FfiNetworkMode::Relay,
        }
    }
}
