use super::{ComicError, RpcError};

/// Ponte entre os erros internos deste crate e o `ConnectionError` que
/// `acerola_p2p::api::protocol::Handler::handle` exige como retorno. Não há um
/// variant específico pra "erro de protocolo de aplicação" na lib, então usamos
/// o balde genérico `StreamFailed` — o mesmo que o RPC interno da lib já usa.
impl From<RpcError> for acerola_p2p::api::error::P2pError {
    fn from(err: RpcError) -> Self {
        acerola_p2p::api::error::P2pError::StreamFailed(err.to_string())
    }
}

impl From<ComicError> for acerola_p2p::api::error::P2pError {
    fn from(err: ComicError) -> Self {
        acerola_p2p::api::error::P2pError::StreamFailed(err.to_string())
    }
}
