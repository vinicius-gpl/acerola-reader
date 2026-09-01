use iroh::{
    endpoint::{
        BindError as IrohBindError, ConnectError as IrohConnectError,
        ConnectingError as IrohConnectingError, ConnectionError as IrohConnectionError,
    },
    RelayUrlParseError,
};

use super::ConnectionError;

impl From<IrohBindError> for ConnectionError {
    fn from(err: IrohBindError) -> Self {
        match err {
            IrohBindError::Sockets { meta, .. } => {
                tracing::debug!(layer = "iroh_transport", ?meta, "failed to bind sockets");
                ConnectionError::StartupFailed("port unavailable".into())
            },
            IrohBindError::CreateQuicEndpoint { meta, .. } => {
                tracing::debug!(layer = "iroh_transport", ?meta, "failed to create QUIC endpoint");
                ConnectionError::StartupFailed("failed to create QUIC endpoint".into())
            },
            IrohBindError::CreateNetmonMonitor { meta, .. } => {
                tracing::debug!(
                    layer = "iroh_transport",
                    ?meta,
                    "failed to create network monitor"
                );
                ConnectionError::StreamFailed("network monitor unavailable".into())
            },
            IrohBindError::InvalidTransportConfig { meta, .. } => {
                tracing::debug!(layer = "iroh_transport", ?meta, "invalid transport configuration");
                ConnectionError::StartupFailed("invalid transport configuration".into())
            },
            IrohBindError::InvalidCaRootConfig { meta, .. } => {
                tracing::debug!(
                    layer = "iroh_transport",
                    ?meta,
                    "invalid certificate configuration"
                );
                ConnectionError::StartupFailed("invalid certificate configuration".into())
            },
            err => {
                tracing::debug!(layer = "iroh_transport", error = ?err, "unmapped bind error");
                ConnectionError::StreamFailed(err.to_string())
            },
        }
    }
}

impl From<IrohConnectError> for ConnectionError {
    fn from(err: IrohConnectError) -> Self {
        match err {
            IrohConnectError::Connection { source, .. } => ConnectionError::from(source),
            IrohConnectError::Connect { meta, .. } => {
                tracing::debug!(layer = "iroh_transport", ?meta, "failed to initiate connection");
                ConnectionError::PeerDisconnected("failed to initiate connection".into())
            },
            IrohConnectError::Connecting { meta, .. } => {
                tracing::debug!(layer = "iroh_transport", ?meta, "handshake failed");
                ConnectionError::PeerDisconnected("handshake failed".into())
            },
            err => {
                tracing::debug!(layer = "iroh_transport", error = ?err, "unmapped connect error");
                ConnectionError::StreamFailed(err.to_string())
            },
        }
    }
}

impl From<IrohConnectingError> for ConnectionError {
    fn from(err: IrohConnectingError) -> Self {
        match err {
            IrohConnectingError::ConnectionError { source, .. } => ConnectionError::from(source),
            IrohConnectingError::LocallyRejected { .. } => {
                tracing::warn!(layer = "iroh_transport", "connection rejected locally by guard");
                ConnectionError::AuthDenied("rejected locally by guard".into())
            },
            IrohConnectingError::HandshakeFailure { .. } => {
                tracing::warn!(
                    layer = "iroh_transport",
                    "handshake failed — invalid or untrusted peer"
                );
                ConnectionError::AuthDenied("invalid or untrusted peer".into())
            },
            IrohConnectingError::InternalConsistencyError { .. } => {
                tracing::debug!(layer = "iroh_transport", "internal consistency error");
                ConnectionError::StreamFailed("internal error".into())
            },
            err => {
                tracing::debug!(layer = "iroh_transport", error = ?err, "unmapped connecting error");
                ConnectionError::StreamFailed(err.to_string())
            },
        }
    }
}

/// Classifica um `iroh::endpoint::ConnectionError` (o mesmo tipo, "noq" por baixo — ver
/// `Cargo.lock`) em `ConnectionError` — extraído do `From` logo abaixo pra também ser
/// reaproveitado por `infra/error/iroh_blobs.rs::classify_get_error`, que precisa da MESMA
/// classificação mas a partir de uma referência emprestada (downcast de um `io::Error`
/// genérico, sem posse do valor original) — daí receber `&IrohConnectionError` em vez de
/// consumir por valor.
pub(super) fn classify_connection_error(err: &IrohConnectionError) -> ConnectionError {
    match err {
        IrohConnectionError::TimedOut => {
            tracing::warn!(layer = "iroh_transport", "connection timed out");
            ConnectionError::Timeout
        },
        IrohConnectionError::Reset => {
            tracing::warn!(layer = "iroh_transport", "connection reset by peer");
            ConnectionError::PeerDisconnected("connection reset by peer".into())
        },
        IrohConnectionError::ConnectionClosed(_) => {
            tracing::debug!(layer = "iroh_transport", "connection closed by peer");
            ConnectionError::PeerDisconnected("connection closed by peer".into())
        },
        IrohConnectionError::ApplicationClosed(_) => {
            tracing::debug!(layer = "iroh_transport", "connection closed by application");
            ConnectionError::PeerDisconnected("connection closed by application".into())
        },
        IrohConnectionError::VersionMismatch => {
            tracing::warn!(layer = "iroh_transport", "incompatible protocol version");
            ConnectionError::IncompatibleVersion
        },
        IrohConnectionError::LocallyClosed => {
            tracing::debug!(layer = "iroh_transport", "connection closed locally");
            ConnectionError::Shutdown
        },
        err => {
            tracing::debug!(layer = "iroh_transport", error = ?err, "unmapped connection error");
            ConnectionError::StreamFailed(err.to_string())
        },
    }
}

impl From<IrohConnectionError> for ConnectionError {
    fn from(err: IrohConnectionError) -> Self {
        classify_connection_error(&err)
    }
}

impl From<RelayUrlParseError> for ConnectionError {
    fn from(relay_err: RelayUrlParseError) -> Self {
        tracing::debug!(
            layer = "iroh_transport",
            error = ?relay_err,
            "failed to parse relay URL"
        );

        ConnectionError::StartupFailed("invalid relay URL".into())
    }
}
