//! Leitura/escrita de frames length-delimited (JSON) compartilhada por todos os protocolos P2P
//! (files/sync-comic, history, library-browse, cover-browse). Antes desta consolidação, cada um
//! reimplementava a MESMA lógica na mão — quatro cópias quase idênticas, cada uma com seu
//! próprio texto de erro e (sem nenhuma razão documentada) seu próprio valor de timeout — e uma
//! delas classificava o timeout de leitura como `P2pError::StreamFailed` genérico em vez de
//! `P2pError::Timeout`, fazendo a sessão inteira falhar sem NENHUMA tentativa de retry, mesmo
//! quando o problema era um hiccup passageiro (achado ao vivo em produção). Uma única
//! implementação aqui garante que a classificação certa (`Timeout`, reconhecida por
//! `classify_sync_error` e retentada inteira por `handle_connect_command` em `acerola-p2p`)
//! valha pra todo protocolo igual, sem depender de lembrar de replicar o fix em N lugares.

use std::time::Duration;

use acerola_p2p::api::error::P2pError;
use futures::SinkExt;
use serde::{de::DeserializeOwned, Serialize};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

pub(crate) type Recv = FramedRead<Box<dyn AsyncRead + Send + Unpin>, LengthDelimitedCodec>;
pub(crate) type Writer = FramedWrite<Box<dyn AsyncWrite + Send + Unpin>, LengthDelimitedCodec>;

/// Lê um frame cru (bytes), com timeout — cada camada do tipo aninhado que
/// `tokio::time::timeout(_, recv.next())` devolve (`Result<Option<Result<BytesMut, io::Error>>,
/// Elapsed>`) vira uma causa DIFERENTE: estourou o tempo, o stream fechou antes de chegar
/// nada, ou deu erro lendo/decodificando o frame em si. `P2pError::Timeout` (não
/// `StreamFailed`) de propósito só pro primeiro caso — é a variante que `classify_sync_error`
/// já reconhece e que `handle_connect_command` (lib/p2p) já trata como transiente o bastante
/// pra retentar a sessão inteira (novo dial + handler de novo); os outros dois continuam
/// `StreamFailed`, que hoje não é retentado.
pub(crate) async fn read_bytes(recv: &mut Recv, timeout: Duration) -> Result<Vec<u8>, P2pError> {
    let frame = tokio::time::timeout(timeout, recv.next())
        .await
        .map_err(|_| P2pError::Timeout)?
        .ok_or_else(|| P2pError::StreamFailed("stream closed before frame".into()))?
        .map_err(|err| P2pError::StreamFailed(format!("failed to read frame: {err}")))?;

    Ok(frame.to_vec())
}

/// Lê e decodifica um frame JSON, com timeout — mesma classificação de erro de [`read_bytes`].
pub(crate) async fn read_json<T: DeserializeOwned>(
    recv: &mut Recv, timeout: Duration,
) -> Result<T, P2pError> {
    let bytes = read_bytes(recv, timeout).await?;
    serde_json::from_slice(&bytes)
        .map_err(|err| P2pError::StreamFailed(format!("failed to decode frame: {err}")))
}

/// Escreve um valor como frame JSON solto (sem tag de enum) — mesmo formato que o Desktop usa
/// em `infra/sync/framing.rs::write_json`.
pub(crate) async fn write_json<T: Serialize>(
    writer: &mut Writer, value: &T,
) -> Result<(), P2pError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|err| P2pError::StreamFailed(format!("failed to encode frame: {err}")))?;
    writer
        .send(bytes.into())
        .await
        .map_err(|err| P2pError::StreamFailed(format!("failed to write frame: {err}")))
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use tokio_util::codec::{FramedRead, FramedWrite};

    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Ping {
        n: u32,
    }

    fn framed_pair() -> (Writer, Recv) {
        let (client, server) = tokio::io::duplex(1024);
        let writer: Writer = FramedWrite::new(Box::new(client), LengthDelimitedCodec::new());
        let reader: Recv = FramedRead::new(Box::new(server), LengthDelimitedCodec::new());
        (writer, reader)
    }

    #[tokio::test]
    async fn write_json_then_read_json_roundtrips() {
        let (mut writer, mut reader) = framed_pair();

        write_json(&mut writer, &Ping { n: 42 }).await.unwrap();
        let received: Ping = read_json(&mut reader, Duration::from_secs(1)).await.unwrap();

        assert_eq!(received, Ping { n: 42 });
    }

    /// Regressão central: um timeout de leitura precisa virar `P2pError::Timeout` — não
    /// `StreamFailed` — pra `handle_connect_command` (lib/p2p) saber que vale a pena retentar a
    /// sessão inteira.
    #[tokio::test]
    async fn read_json_times_out_as_a_typed_timeout_not_a_generic_stream_failure() {
        let (_writer, mut reader) = framed_pair();

        let result: Result<Ping, P2pError> =
            read_json(&mut reader, Duration::from_millis(20)).await;

        assert!(
            matches!(result, Err(P2pError::Timeout)),
            "esperava P2pError::Timeout, recebeu {result:?}"
        );
    }

    /// Caminho triste diferente do timeout: o outro lado fechou o stream de propósito (ou
    /// caiu) ANTES de mandar qualquer frame — isso é `StreamFailed`, não `Timeout`, porque
    /// retentar imediatamente não muda nada (a conexão já não existe mais).
    #[tokio::test]
    async fn read_json_reports_a_clean_stream_close_as_stream_failed_not_timeout() {
        let (writer, mut reader) = framed_pair();
        drop(writer);

        let result: Result<Ping, P2pError> =
            read_json(&mut reader, Duration::from_secs(1)).await;

        assert!(
            matches!(result, Err(P2pError::StreamFailed(_))),
            "esperava StreamFailed (stream fechado), recebeu {result:?}"
        );
    }

    #[tokio::test]
    async fn read_json_reports_malformed_json_as_stream_failed() {
        let (mut writer, mut reader) = framed_pair();

        writer.send(b"not valid json".to_vec().into()).await.unwrap();

        let result: Result<Ping, P2pError> =
            read_json(&mut reader, Duration::from_secs(1)).await;

        assert!(
            matches!(result, Err(P2pError::StreamFailed(_))),
            "esperava StreamFailed (JSON malformado), recebeu {result:?}"
        );
    }
}
