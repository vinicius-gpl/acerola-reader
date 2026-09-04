use super::{iroh::classify_connection_error, ConnectionError};

/// `iroh_blobs::api::Error` já centraliza os erros de RPC/IO internos do `iroh-blobs`
/// (conexões RPC quebradas, falhas de I/O em disco) num único wrapper — não precisamos de um
/// `match` por variante como em `iroh.rs`, só reportar e mapear pro caso genérico de stream.
impl From<iroh_blobs::api::Error> for ConnectionError {
    fn from(err: iroh_blobs::api::Error) -> Self {
        tracing::debug!(layer = "iroh_blobs", error = ?err, "blob store operation failed");
        ConnectionError::StreamFailed(err.to_string())
    }
}

impl From<iroh_blobs::api::RequestError> for ConnectionError {
    fn from(err: iroh_blobs::api::RequestError) -> Self {
        ConnectionError::from(iroh_blobs::api::Error::from(err))
    }
}

/// Tenta recuperar uma classificação estruturada (não texto) de um `GetError` — ele não expõe
/// as causas de rede como variantes próprias, mas embrulha o `io::Error` real da stream QUIC
/// (`ReadError`/`WriteError`/`ConnectionError` do `iroh::endpoint`) atrás de `remote_read()`/
/// `remote_write()`/`open()`/`local_write()`. Sem isso, TODO fetch de blob que falha por rede —
/// timeout, peer resetando a stream, conexão caindo — virava o mesmo `StreamFailed(err.to_string())`
/// genérico, e quem consumia esse erro tinha que adivinhar a causa fazendo substring matching em
/// cima do texto (frágil: quebra se a lib mudar a mensagem, e não distingue "timed out" de
/// "reset by peer" de forma confiável). Downcast pro tipo real e reaproveita o mesmo `match` já
/// usado pra conexões normais (`classify_connection_error`, extraído de `iroh.rs`) — mesmo
/// código, texto Display idêntico, só a origem do erro é diferente (conexão dedicada de
/// `fetch()` vs. o transporte principal). Retorna `None` pra causas que não têm um paralelo
/// estruturado (erro de decodificação do protocolo de blob, I/O local em disco etc.) — esses
/// continuam caindo no fallback genérico de `StreamFailed` no `From` abaixo.
fn classify_get_error(err: &iroh_blobs::get::GetError) -> Option<ConnectionError> {
    let io_err = err
        .remote_read()
        .or_else(|| err.remote_write())
        .or_else(|| err.open())
        .or_else(|| err.local_write())?;
    let source = io_err.get_ref()?;

    if let Some(read_err) = source.downcast_ref::<iroh::endpoint::ReadError>() {
        return match read_err {
            iroh::endpoint::ReadError::Reset(_) => {
                tracing::warn!(layer = "iroh_blobs", "stream reset by peer during blob fetch");
                Some(ConnectionError::PeerDisconnected("stream reset by peer".into()))
            },
            iroh::endpoint::ReadError::ConnectionLost(conn_err) => {
                Some(classify_connection_error(conn_err))
            },
            _ => None,
        };
    }
    if let Some(write_err) = source.downcast_ref::<iroh::endpoint::WriteError>() {
        return match write_err {
            iroh::endpoint::WriteError::Stopped(_) => {
                tracing::warn!(
                    layer = "iroh_blobs",
                    "peer stopped accepting stream data during blob fetch"
                );
                Some(ConnectionError::PeerDisconnected("peer stopped accepting stream data".into()))
            },
            iroh::endpoint::WriteError::ConnectionLost(conn_err) => {
                Some(classify_connection_error(conn_err))
            },
            _ => None,
        };
    }
    if let Some(conn_err) = source.downcast_ref::<iroh::endpoint::ConnectionError>() {
        return Some(classify_connection_error(conn_err));
    }

    None
}

/// Erro de uma requisição `get` (usado por `Remote::fetch`) — tipo próprio do `iroh-blobs`,
/// não convertível para `api::Error` porque cobre falhas específicas do protocolo de streaming
/// (decodificação, handshake de blob) além de I/O puro.
impl From<iroh_blobs::get::GetError> for ConnectionError {
    fn from(err: iroh_blobs::get::GetError) -> Self {
        if let Some(classified) = classify_get_error(&err) {
            return classified;
        }
        tracing::debug!(layer = "iroh_blobs", error = ?err, "blob fetch from peer failed");
        ConnectionError::StreamFailed(err.to_string())
    }
}

/// Erro de baixo nível do transporte RPC interno do `iroh-blobs` (`store.tags()`/`store.blobs()`
/// se comunicam com o ator do store via `irpc`).
impl From<irpc::Error> for ConnectionError {
    fn from(err: irpc::Error) -> Self {
        tracing::debug!(layer = "iroh_blobs", error = ?err, "blob store rpc failed");
        ConnectionError::StreamFailed(err.to_string())
    }
}

/// Erro genérico do `n0-error` (framework de erros do ecossistema n0/iroh) — usado por
/// `FsStore::load_with_opts` ao abrir/criar o store em disco. Falha aqui é sempre um problema de
/// inicialização (ex: diretório sem permissão), nunca de uma stream já em andamento.
impl From<n0_error::AnyError> for ConnectionError {
    fn from(err: n0_error::AnyError) -> Self {
        tracing::debug!(layer = "iroh_blobs", error = ?err, "blob store initialization failed");
        ConnectionError::StartupFailed(err.to_string())
    }
}
