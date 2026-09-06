use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use acerola_p2p::api::{error::P2pError, peer::PeerIdentity, protocol::EventEmitter};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite},
    sync::Semaphore,
    task::JoinSet,
};

use crate::{
    core::services::{metadata::MetadataService, sync::file_sync::FileSyncService},
    infra::{
        error::RpcError,
        sync::{
            blob_context::BlobContext,
            framing::{framed_writer, read_json, write_json, FramedReader, FramedWriter},
            messages::{FileExtraHeader, FileHeader, SessionBusy, EXTRA_KIND_COMIC_INFO},
        },
    },
};

/// Publica/busca os bytes de um capítulo via `P2pBlobStore` (BLAKE3, dedup e verificação
/// automáticos), independente do stream de controle (`FramedReader`/`FramedWriter`) que só
/// carrega o `FileHeader` com o hash. Trait (em vez de `Arc<BlobContext>` direto) só pra manter
/// os testes de `send_files`/`receive_files` sem depender de um `Endpoint` iroh de verdade —
/// `BlobChapterTransfer` é a única implementação usada em produção.
#[async_trait]
pub trait ChapterTransfer: Send + Sync {
    async fn publish(&self, bytes: Vec<u8>) -> Result<String, P2pError>;

    async fn fetch_reader(
        &self, blob_hash: &str, peer: &PeerIdentity,
    ) -> Result<Box<dyn AsyncRead + Send + Unpin>, P2pError>;

    /// RTT medido com `peer` agora, usado só pra dimensionar quantos capítulos buscar em
    /// paralelo em `receive_files` (ver doc lá) — `None` quando não há como medir (sem conexão
    /// viva, ou implementação de teste que não simula rede de verdade) cai no paralelismo
    /// mínimo, nunca em erro.
    async fn latency(&self, peer: &PeerIdentity) -> Option<std::time::Duration>;
}

pub struct BlobChapterTransfer {
    context: Arc<BlobContext>,
}

impl BlobChapterTransfer {
    pub fn new(context: Arc<BlobContext>) -> Self {
        Self { context }
    }
}

#[async_trait]
impl ChapterTransfer for BlobChapterTransfer {
    async fn publish(&self, bytes: Vec<u8>) -> Result<String, P2pError> {
        let store = self.context.blob_store().await?;
        // `P2pBlobStore::put` já retorna `ConnectionError` (= `P2pError`, mesmo tipo — ver
        // `acerola_p2p::api::error`) — sem o `.map_err(|err| P2pError::StreamFailed(err.to_string()))`
        // que tinha aqui antes, o `?` propaga a variante de verdade (`Timeout`,
        // `PeerDisconnected`, etc.) direto pro chamador. Re-envelopar tudo em `StreamFailed`
        // achatava essa classificação já corrigida na origem (ver `classify_get_error` em
        // `lib/p2p/src/infra/error/iroh_blobs.rs`) de volta em texto cru, obrigando quem
        // consumia esse erro a adivinhar a causa fazendo substring matching no `to_string()`.
        let hash = store.put(bytes).await?;
        Ok(hash.to_string())
    }

    async fn fetch_reader(
        &self, blob_hash: &str, peer: &PeerIdentity,
    ) -> Result<Box<dyn AsyncRead + Send + Unpin>, P2pError> {
        let store = self.context.blob_store().await?;
        // Formato do hash é local/estático (não vem da rede) — não tem paralelo em
        // `ConnectionError` além do genérico `StreamFailed`, então mantém `.map_err` só aqui.
        let hash = blob_hash
            .parse()
            .map_err(|_| P2pError::StreamFailed(format!("invalid blob hash: {blob_hash}")))?;
        let addr = self.context.resolve_addr(peer).await?;

        // Ver comentário em `publish` — `fetch`/`get` já retornam `ConnectionError` estruturado,
        // propaga direto em vez de achatar em `StreamFailed(err.to_string())`.
        store.fetch(&hash, &addr).await?;
        store.get(&hash).await
    }

    async fn latency(&self, peer: &PeerIdentity) -> Option<std::time::Duration> {
        self.context.latency(peer).await
    }
}

/// Texto de rejeição usado tanto no evento local (`emit_busy`) quanto na mensagem `SessionBusy`
/// escrita no wire (`write_session_busy`) — mesmo motivo nos dois lugares. Compartilhado entre
/// `acerola/sync-files/1` e `acerola/sync-comic/1`: os dois competem pelo mesmo
/// `FileSyncSessionGuard` por peer (ver `file_session_guard.rs`).
pub const SESSION_BUSY_REASON: &str = "file sync session already in progress for this peer";

/// Valor do campo `error` em `SessionBusy` — mesmo literal usado pelo Android
/// (`protocol/files/exchange.rs::SESSION_BUSY_TAG`), é o que os dois lados usam pra reconhecer
/// essa mensagem específica em vez de um `FileManifest` normal.
pub(super) const SESSION_BUSY_TAG: &str = "busy";

/// Motivo usado quando o lado outbound de um sync com escopo prévio (`sync-comic`,
/// `browse-cover`) dispara a conexão sem ter registrado a intenção local antes — só acontece se
/// o comando Tauri esqueceu de popular o registry (`PendingComicSyncRegistry`/
/// `PendingCoverRequestRegistry`) antes de chamar `connect()`. Compartilhado entre os dois
/// handlers pelo mesmo motivo de `SESSION_BUSY_REASON`: uma string nossa reconhecida por
/// igualdade exata em `classify_sync_error`, não heurística de texto de terceiros — e
/// compartilhada entre os dois sites pra não divergir (antes cada handler tinha sua própria
/// frase, nenhuma delas classificada, então a UI sempre mostrava o texto cru em inglês).
pub const NO_PENDING_SCOPE_REASON: &str = "no pending sync scope registered for this peer";

/// Identificador estável de causa de erro de sync, serializado em `snake_case` (`"busy"`,
/// `"timeout"`, `"connection_lost"`) — o mesmo valor que o frontend já esperava quando isso era
/// uma `&'static str` solta, então trocar pra enum não muda o contrato de wire (`SYNC_ERROR_MESSAGES`
/// em `use-network-sync.svelte.ts` continua igual). A vantagem do enum é o `match` em
/// `classify_sync_error` ser exaustivo e verificado em tempo de compilação — sem isso, cada causa
/// nova era "só mais um `.contains()`" testando texto arbitrário de uma lib externa, frágil (quebra
/// se a mensagem mudar) e sem nenhuma garantia de estar cobrindo os casos reais.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum SyncErrorCode {
    Busy,
    Timeout,
    ConnectionLost,
    PeerNotFound,
    AuthDenied,
    Shutdown,
    IncompatibleVersion,
    StartupFailed,
    BlobNotFound,
    /// Header recebido sem `blob_hash` — não deveria acontecer (bug de protocolo, não falha de
    /// rede), mas ainda precisa de um código traduzível em vez de cair no texto cru.
    MissingBlobHash,
    /// Fallback genérico pra falha de `ChapterTransfer::fetch_reader` quando o `P2pError`
    /// subjacente não é nenhuma das causas específicas já cobertas acima.
    BlobFetchFailed,
    ChecksumMismatch,
    ComicDirectoryUnavailable,
    PersistFailed,
    NoPendingRequest,
    /// Sessão terminou sem erro de protocolo, mas nem todos os itens chegaram (ver doc de
    /// `receive_files`/`receive_extras`) — antes essas duas ocorrências (`comic_handler.rs`,
    /// `file_handler.rs`) montavam a mensagem já em pt-BR direto no backend, sem `code`
    /// nenhum: violava a regra de erro tipado por dois lados ao mesmo tempo (texto fixo numa
    /// língua Y decidido no backend, E sem tradução nenhuma pro frontend usar).
    PartialSync,
}

/// Classifica um `P2pError` (não seu texto) num `SyncErrorCode` que o frontend traduz via
/// Paraglide (`SYNC_ERROR_MESSAGES` em `use-network-sync.svelte.ts`). Isso só é possível de
/// forma confiável porque `ChapterTransfer::publish`/`fetch_reader` (`BlobChapterTransfer`, logo
/// acima) pararam de achatar `ConnectionError` em `StreamFailed(err.to_string())` — o erro que
/// chega aqui já é a variante de verdade que `lib/p2p` classificou na origem
/// (`classify_connection_error`/`classify_get_error`, ver `infra/error/{iroh,iroh_blobs}.rs` no
/// crate `acerola-p2p`), não texto pra adivinhar via substring matching. `Busy` continua sendo a
/// exceção: não é uma variante de `ConnectionError` (é um conceito deste protocolo de sync, não
/// do transporte), então a única forma de reconhecê-la através do `Result<(), P2pError>` fixo de
/// `Handler::handle` é o texto que ESTE MÓDULO controla (`SESSION_BUSY_REASON`) — igualdade
/// exata numa constante nossa, não uma heurística sobre texto de terceiros.
pub(super) fn classify_sync_error(error: &P2pError) -> Option<SyncErrorCode> {
    match error {
        P2pError::Timeout => Some(SyncErrorCode::Timeout),
        P2pError::PeerDisconnected(_) => Some(SyncErrorCode::ConnectionLost),
        P2pError::StreamFailed(msg) if msg.contains(SESSION_BUSY_REASON) => {
            Some(SyncErrorCode::Busy)
        },
        P2pError::StreamFailed(msg) if msg == NO_PENDING_SCOPE_REASON => {
            Some(SyncErrorCode::NoPendingRequest)
        },
        // Resto de `StreamFailed` é texto de I/O genérico de baixo nível (framing, disco) sem
        // uma causa específica pra nomear — cai no fallback de quem chamou (`message` cru no
        // log do backend, que nunca é mostrado como está na UI).
        P2pError::StreamFailed(_) => None,
        P2pError::PeerNotFound(_) => Some(SyncErrorCode::PeerNotFound),
        P2pError::AuthDenied(_) => Some(SyncErrorCode::AuthDenied),
        P2pError::Shutdown => Some(SyncErrorCode::Shutdown),
        P2pError::IncompatibleVersion => Some(SyncErrorCode::IncompatibleVersion),
        P2pError::StartupFailed(_) => Some(SyncErrorCode::StartupFailed),
        P2pError::BlobNotFound(_) => Some(SyncErrorCode::BlobNotFound),
    }
}

/// Emitido quando `FileSyncSessionGuard` recusa uma sessão porque já existe uma ativa pro
/// mesmo peer — reaproveita o evento `sync:files:error` já tratado pelo frontend em vez de
/// introduzir um evento novo (ver `use-network-sync.svelte.ts::parseErrorPayload`). O chamador
/// passa o nome do evento (`sync:files:error` ou `sync:comic:error`) porque cada protocolo tem
/// o seu próprio canal. `message` continua em inglês (é texto técnico de log/wire, não deve ser
/// mostrado como está) — `code` é o identificador estável que o frontend usa pra traduzir via
/// Paraglide (`tauri_errors.sync.session_busy.label`), no mesmo padrão de `errors.i18n.ts`.
pub fn emit_busy(emit: &EventEmitter, event_name: &str, peer_id: &str) {
    (emit)(
        event_name,
        sync_error_payload(peer_id, SESSION_BUSY_REASON, Some(SyncErrorCode::Busy), None),
    );
}

/// Monta o JSON de `sync:*:error` usado tanto por `emit_busy` quanto pelo branch `Err(error)`
/// genérico de `FileSyncOutbound`/`FileSyncInbound`/`ComicSyncOutbound`/`ComicSyncInbound`
/// (`file_handler.rs`/`comic_handler.rs`) — um lugar só pro shape do payload em vez de 5
/// `serde_json::json!({...})` quase-iguais espalhados, que já divergiam entre si (nem todos
/// incluíam `comicName`). `code: None` serializa como `null` — o frontend trata isso como "sem
/// tradução conhecida, mostra o `message` cru" (`translateSyncMessage`).
pub(super) fn sync_error_payload(
    peer_id: &str, message: &str, code: Option<SyncErrorCode>, comic_name: Option<&str>,
) -> String {
    serde_json::json!({
        "peerId": peer_id,
        "message": message,
        "code": code,
        "comicName": comic_name,
    })
    .to_string()
}

/// Variante de `sync_error_payload` pros 5 pontos de falha POR ITEM em `receive_files`/
/// `receive_extras` (blob não encontrado, fetch falhou, checksum não bate, pasta indisponível,
/// persistência falhou) — todos não-fatais: o item é pulado (`skipped += 1`) e o loop continua
/// pro próximo capítulo/extra, a sessão inteira não aborta.
///
/// Sem `"itemLevel": true` + `"item"` aqui, o frontend não tinha como distinguir esses eventos
/// de um erro de sessão de verdade (`Err(error)` em `handle()`, ou `PartialSync`) — os dois
/// carregam o mesmo `peerId`, então `use-network-sync.svelte.ts` correlacionava QUALQUER
/// `sync:files:error`/`sync:comic:error` com a linha "started"/"progress" em andamento daquele
/// peer e marcava como terminal — uma sessão de 20 capítulos com 1 checksum mismatch no meio
/// mostrava "erro" na tela mesmo continuando (e completando) normalmente depois. Android já
/// resolvia isso do lado dele com um sentinela equivalente (`comicName`/`chapter` vazios =
/// falha de sessão inteira, ver `protocol::files::run_and_report` no crate nativo) — este helper
/// traz o mesmo contrato pro Desktop, explícito em vez de inferido por um sentinela silencioso.
pub(super) fn item_error_payload(
    peer_id: &str, message: &str, code: Option<SyncErrorCode>, comic_name: &str, item: &str,
) -> String {
    serde_json::json!({
        "peerId": peer_id,
        "message": message,
        "code": code,
        "comicName": comic_name,
        "item": item,
        "itemLevel": true,
    })
    .to_string()
}

/// Escreve a mensagem de rejeição de sessão diretamente no stream, no lugar do manifesto —
/// usado quando `FileSyncSessionGuard::try_acquire` já barrou a sessão em `handle()`, antes do
/// resto do protocolo rodar. Recebe `send` cru (não um `FramedWriter` já montado) porque é
/// exatamente o estado em que `handle()` está no momento da rejeição: o stream ainda não foi
/// envolvido em nenhum framing. Melhor esforço: se a escrita falhar (peer já caiu, stream já
/// fechado), ignora — a sessão já foi rejeitada localmente de qualquer forma.
pub async fn write_session_busy(send: Box<dyn AsyncWrite + Send + Unpin>) {
    let mut writer = framed_writer(send);
    let _ = write_json(
        &mut writer,
        &SessionBusy {
            error: SESSION_BUSY_TAG.to_string(),
            reason: SESSION_BUSY_REASON.to_string(),
        },
    )
    .await;
}

/// Lê a primeira mensagem do peer, que pode ser tanto a mensagem esperada (`T`: `FileManifest`
/// inteiro ou escopado a um único quadrinho — ver `FileSyncService::build_manifest_for_comic`
/// — ou `ComicSyncRequest` no protocolo `acerola/sync-comic/1`) quanto uma rejeição
/// `SessionBusy` — sem tag de enum no wire (mesma convenção do resto do protocolo), a única
/// forma de saber qual é espiar o JSON bruto antes de decidir o tipo concreto. Só o(s) ponto(s)
/// de leitura inicial em cada `run()` usa(m) isso: depois da primeira mensagem, uma sessão
/// aceita nunca mais manda `SessionBusy`.
pub async fn read_or_busy<T: DeserializeOwned>(reader: &mut FramedReader) -> Result<T, P2pError> {
    let value: serde_json::Value = read_json(reader).await?;

    let is_busy = value.get("error").and_then(|tag| tag.as_str()) == Some(SESSION_BUSY_TAG);
    if is_busy {
        let reason = value.get("reason").and_then(|r| r.as_str()).unwrap_or("peer is busy");
        return Err(P2pError::StreamFailed(format!("peer busy: {reason}")));
    }

    serde_json::from_value(value).map_err(|err| RpcError::Deserialize(err.to_string()).into())
}

/// Envia, em sequência, os arquivos de `wanted` (o que o peer pediu de nós). Se um item não
/// existir mais localmente (corrida com uma deleção concorrente), envia um header
/// "indisponível" (`size: 0`) em vez de abortar a sessão inteira. Os bytes não trafegam mais
/// no stream de controle — são publicados no blob store local (`ChapterTransfer::publish`) e o
/// hash resultante vai no `FileHeader`; quem recebe busca sob demanda via `fetch_reader`.
///
/// Retorna quantos itens de `wanted` NÃO foram enviados de verdade (arquivo sumiu localmente
/// entre o want-list e o envio) — o chamador usa essa contagem pra decidir se a sessão pode
/// mesmo ser reportada como "completa" pro usuário (ver `ComicSyncOutbound`/`FileSyncOutbound`).
pub async fn send_files(
    writer: &mut FramedWriter, wanted: &[(String, String)], service: &FileSyncService,
    emit: &EventEmitter, progress_event: &str, transfer: &Arc<dyn ChapterTransfer>,
) -> Result<usize, P2pError> {
    let mut skipped = 0usize;

    for (comic_name, chapter) in wanted {
        let resolved = service.resolve_local_file(comic_name, chapter).await?;

        let Some((path, size, checksum, file_name)) = resolved else {
            tracing::warn!(
                comic_name = %comic_name,
                chapter = %chapter,
                "[FileSync] requested file no longer exists locally — sending unavailable header"
            );
            skipped += 1;
            write_json(
                writer,
                &FileHeader {
                    comic_name: comic_name.clone(),
                    chapter: chapter.clone(),
                    file_name: String::new(),
                    size: 0,
                    checksum: None,
                    blob_hash: None,
                },
            )
            .await?;
            continue;
        };

        let bytes = tokio::fs::read(&path).await.map_err(RpcError::from)?;
        let blob_hash = transfer.publish(bytes).await?;

        write_json(
            writer,
            &FileHeader {
                comic_name: comic_name.clone(),
                chapter: chapter.clone(),
                file_name,
                size,
                checksum,
                blob_hash: Some(blob_hash),
            },
        )
        .await?;

        (emit)(progress_event, format!("{} - {}", comic_name, chapter));
    }

    Ok(skipped)
}

/// Quantos capítulos buscar em paralelo, a partir do RTT medido AGORA com o peer (ver
/// `ChapterTransfer::latency`) — um link lento (relay distante) precisa de vários fetches "em
/// voo" ao mesmo tempo pra não ficar ocioso entre um round-trip e o próximo; um link local
/// (mDNS/LAN) já satura com pouquíssimo paralelismo, e mais que isso só desperdiça memória.
/// Teto fixo de 6 de propósito: uma rajada de fetches sem limite nenhum (`fetchCoversFor`, no
/// frontend) foi exatamente o que deixou uma conexão real degradada a ponto de um sync de
/// arquivos levar minutos, capítulo a capítulo — não faz sentido resolver "devagar demais"
/// recriando "rápido demais a ponto de quebrar" pro mesmo peer. Sem medição (peer ainda não
/// respondeu a nenhum ping, ou implementação de teste) cai no mais conservador dos três — não
/// vale a pena arriscar paralelismo sobre uma conexão de saúde desconhecida.
fn chapter_fetch_concurrency(latency: Option<std::time::Duration>) -> usize {
    match latency {
        Some(rtt) if rtt <= std::time::Duration::from_millis(20) => 2,
        Some(rtt) if rtt <= std::time::Duration::from_millis(200) => 4,
        Some(_) => 6,
        None => 2,
    }
}

/// Quantos permits tirar do pool (`Semaphore::forget_permits`) na primeira vez que um fetch
/// estoura o timeout de inatividade numa sessão — corta pela metade, nunca abaixo de 1 (nunca
/// desliga o paralelismo por completo, só reduz). Calculado a partir do N ORIGINAL
/// (`initial_permits` em `receive_files`), não de `available_permits()` em tempo real: no
/// momento do primeiro timeout a maioria do N original já está em uso por fetches em voo, então
/// `available_permits()` reflete só a sobra e subestimaria o corte.
fn permits_to_forget(initial_permits: usize) -> usize {
    initial_permits.saturating_sub((initial_permits / 2).max(1))
}

/// Resultado de processar UM header já lido do fio — extraído de `receive_files` pra poder
/// rodar como uma task independente (`tokio::spawn`), em paralelo com a leitura do PRÓXIMO
/// header, em vez de bloquear o loop inteiro no fetch de um único capítulo por vez.
enum ChapterOutcome {
    Persisted,
    /// `timed_out: true` só quando a causa específica de pular este item foi
    /// `P2pError::Timeout` no fetch do blob — o único sinal que `receive_files` usa pra reduzir
    /// o paralelismo pro resto da sessão (ver doc lá). Os outros motivos de skip (blob_hash
    /// ausente, checksum, pasta indisponível, falha de persistência, falha de leitura local do
    /// blob já fetchado) não indicam uma conexão degradada, só um item ruim isolado — não faz
    /// sentido isso disparar o backoff.
    Skipped {
        timed_out: bool,
    },
}

#[allow(clippy::too_many_arguments)]
async fn receive_one_chapter(
    header: FileHeader, service: FileSyncService, emit: EventEmitter, progress_event: String,
    error_event: String, peer: PeerIdentity, transfer: Arc<dyn ChapterTransfer>,
) -> ChapterOutcome {
    if header.size == 0 {
        tracing::warn!(
            comic_name = %header.comic_name,
            chapter = %header.chapter,
            "[FileSync] peer reported chapter as unavailable (size=0) — skipping"
        );
        return ChapterOutcome::Skipped { timed_out: false };
    }

    let Some(blob_hash) = header.blob_hash.as_deref() else {
        tracing::warn!(
            comic_name = %header.comic_name,
            chapter = %header.chapter,
            "[FileSync] header missing blob_hash — skipping"
        );
        (emit)(
            &error_event,
            item_error_payload(
                &peer.id,
                "missing blob hash",
                Some(SyncErrorCode::MissingBlobHash),
                &header.comic_name,
                &header.chapter,
            ),
        );
        return ChapterOutcome::Skipped { timed_out: false };
    };

    let mut blob_reader = match transfer.fetch_reader(blob_hash, &peer).await {
        Ok(reader) => reader,
        Err(err) => {
            tracing::warn!(
                comic_name = %header.comic_name,
                chapter = %header.chapter,
                error = %err,
                "[FileSync] failed to fetch chapter blob — skipping"
            );
            let timed_out = matches!(err, P2pError::Timeout);
            (emit)(
                &error_event,
                item_error_payload(
                    &peer.id,
                    &format!("blob fetch failed: {err}"),
                    Some(classify_sync_error(&err).unwrap_or(SyncErrorCode::BlobFetchFailed)),
                    &header.comic_name,
                    &header.chapter,
                ),
            );
            return ChapterOutcome::Skipped { timed_out };
        },
    };

    // Antes usava `?` e abortava a sessão INTEIRA — diferente dos outros pontos de falha desta
    // função, que já eram todos não-fatais. Essa leitura é local (`P2pBlobStore::get`, disco/
    // cache já preenchido por `fetch` alguns milissegundos antes), não uma segunda rodada de
    // rede: uma falha aqui não indica nada sobre a saúde da conexão nem do resto do header
    // stream, então tratar como mais um item pulado (em vez de fatal) alinha esta última exceção
    // com a filosofia já estabelecida no resto da função.
    let mut bytes = Vec::new();
    if let Err(err) = blob_reader.read_to_end(&mut bytes).await {
        tracing::warn!(
            comic_name = %header.comic_name,
            chapter = %header.chapter,
            error = %err,
            "[FileSync] failed to read fetched chapter blob — skipping"
        );
        (emit)(
            &error_event,
            item_error_payload(
                &peer.id,
                &format!("failed to read chapter blob: {err}"),
                None,
                &header.comic_name,
                &header.chapter,
            ),
        );
        return ChapterOutcome::Skipped { timed_out: false };
    }

    let computed_checksum = {
        use sha2::{Digest, Sha256};
        format!("{:x}", Sha256::digest(&bytes))
    };

    if let Some(expected) = &header.checksum {
        if expected != &computed_checksum {
            tracing::warn!(
                comic_name = %header.comic_name,
                chapter = %header.chapter,
                expected = %expected,
                computed = %computed_checksum,
                "[FileSync] checksum mismatch — skipping"
            );
            (emit)(
                &error_event,
                item_error_payload(
                    &peer.id,
                    "checksum mismatch",
                    Some(SyncErrorCode::ChecksumMismatch),
                    &header.comic_name,
                    &header.chapter,
                ),
            );
            return ChapterOutcome::Skipped { timed_out: false };
        }
    }

    // Resolve a pasta do quadrinho ANTES de gravar o arquivo temporário — grava já no
    // destino final (mesma pasta que `persist_received_chapter` vai usar pro rename),
    // sem uma pasta de staging à parte que sobrava visível na biblioteca do usuário.
    let comic_dir = match service.resolve_comic_dir(&header.comic_name).await {
        Ok(comic) => std::path::PathBuf::from(comic.path),
        Err(err) => {
            tracing::warn!(
                comic_name = %header.comic_name,
                chapter = %header.chapter,
                error = %err,
                "[FileSync] failed to resolve comic directory — skipping"
            );
            (emit)(
                &error_event,
                item_error_payload(
                    &peer.id,
                    "failed to resolve comic directory",
                    Some(SyncErrorCode::ComicDirectoryUnavailable),
                    &header.comic_name,
                    &header.chapter,
                ),
            );
            return ChapterOutcome::Skipped { timed_out: false };
        },
    };
    let temp_path = comic_dir.join(format!(".incoming-{}.tmp", rand::random::<u64>()));
    if let Err(err) = tokio::fs::write(&temp_path, &bytes).await {
        tracing::warn!(
            comic_name = %header.comic_name,
            chapter = %header.chapter,
            error = %err,
            "[FileSync] failed to write temp file for received chapter — skipping"
        );
        (emit)(
            &error_event,
            item_error_payload(
                &peer.id,
                &format!("failed to write temp file: {}", RpcError::from(err)),
                Some(SyncErrorCode::PersistFailed),
                &header.comic_name,
                &header.chapter,
            ),
        );
        return ChapterOutcome::Skipped { timed_out: false };
    }

    if let Err(err) = service
        .persist_received_chapter(
            &header.comic_name,
            &header.chapter,
            &header.file_name,
            &temp_path,
            computed_checksum,
        )
        .await
    {
        tracing::warn!(
            comic_name = %header.comic_name,
            chapter = %header.chapter,
            error = %err,
            "[FileSync] failed to persist received chapter — skipping"
        );
        (emit)(
            &error_event,
            item_error_payload(
                &peer.id,
                &format!("failed to persist chapter: {err}"),
                Some(SyncErrorCode::PersistFailed),
                &header.comic_name,
                &header.chapter,
            ),
        );
        return ChapterOutcome::Skipped { timed_out: false };
    }

    (emit)(&progress_event, format!("{} - {}", header.comic_name, header.chapter));
    ChapterOutcome::Persisted
}

/// Recebe, em sequência, `expected_count` arquivos: cada `FileHeader` traz um `blob_hash` em
/// vez de bytes crus no stream — busca via `ChapterTransfer::fetch_reader` (que já verifica
/// integridade via BLAKE3 antes de devolver o leitor), grava num arquivo temporário e move pro
/// destino final depois de conferir o SHA-256 legado contra o anunciado no header (continuidade
/// do histórico/telemetria existente, redundante com a verificação do blob store mas barato o
/// bastante pra manter).
///
/// Descarta headers "indisponíveis" (`size: 0`), falhas de busca/leitura do blob ou mismatches
/// de checksum sem abortar a sessão inteira — cada caso loga via `tracing` e é contado no
/// retorno (`usize` = quantos capítulos de `expected_count` NÃO foram persistidos). Bug
/// reportado em 22/08/2026: um sync de 2 capítulos persistiu só 1, sem nenhum erro visível no
/// log — porque essas falhas de item nunca logavam nada, só emitiam evento pro frontend, e a
/// sessão inteira ainda era reportada como "completa" pro chamador (`ComicSyncOutbound`/
/// `FileSyncOutbound`) mesmo com itens faltando.
///
/// `#[allow(too_many_arguments)]`: dívida técnica pré-existente (já eram 8 parâmetros antes
/// desta mudança) — agrupar em uma struct de contexto é um refactor maior, fora do escopo do
/// bug que motivou esta função a mudar.
///
/// Cada header é lido em sequência do fio (é um stream ordenado só, não dá pra ler o header 2
/// antes do 1) mas o FETCH do blob que ele referencia roda numa task própria, num pool de até
/// `chapter_fetch_concurrency(...)` capítulos concorrentes — antes disso, o loop bloqueava no
/// fetch inteiro de UM capítulo antes de sequer ler o próximo header, então qualquer lentidão
/// por item (RTT alto, conexão degradada) virava tempo total de sessão 1:1, sem chance de
/// sobrepor um fetch lento com o próximo enquanto ele ainda está em voo. Motivado por um sync
/// real que levou minutos numa conexão de relay de ~1s de RTT, um capítulo por vez.
#[allow(clippy::too_many_arguments)]
pub async fn receive_files(
    reader: &mut FramedReader, expected_count: usize, service: &FileSyncService,
    emit: &EventEmitter, progress_event: &str, error_event: &str, peer: &PeerIdentity,
    transfer: &Arc<dyn ChapterTransfer>,
) -> Result<usize, P2pError> {
    let initial_permits = chapter_fetch_concurrency(transfer.latency(peer).await);
    let semaphore = Arc::new(Semaphore::new(initial_permits));
    // `backed_off` garante que o corte de `permits_to_forget` só dispare uma vez por sessão —
    // ver doc de `permits_to_forget` pro porquê do cálculo.
    let backed_off = Arc::new(AtomicBool::new(false));
    let permits_to_forget_on_timeout = permits_to_forget(initial_permits);

    let mut tasks = JoinSet::new();
    let mut skipped = 0usize;

    for _ in 0..expected_count {
        let header: FileHeader = read_json(reader).await?;

        tracing::debug!(
            comic_name = %header.comic_name,
            chapter = %header.chapter,
            file_name = %header.file_name,
            "[FileSync] header recebido pelo fio"
        );

        let permit = Arc::clone(&semaphore)
            .acquire_owned()
            .await
            .expect("semaphore is never closed while receive_files is running");
        let semaphore_for_backoff = Arc::clone(&semaphore);
        let backed_off = Arc::clone(&backed_off);
        let task_service = service.clone();
        let task_emit = Arc::clone(emit);
        let task_progress_event = progress_event.to_string();
        let task_error_event = error_event.to_string();
        let task_peer = peer.clone();
        let task_transfer = Arc::clone(transfer);

        tasks.spawn(async move {
            let _permit = permit;
            let outcome = receive_one_chapter(
                header,
                task_service,
                task_emit,
                task_progress_event,
                task_error_event,
                task_peer,
                task_transfer,
            )
            .await;

            if matches!(outcome, ChapterOutcome::Skipped { timed_out: true })
                && backed_off
                    .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                && permits_to_forget_on_timeout > 0
            {
                semaphore_for_backoff.forget_permits(permits_to_forget_on_timeout);
            }

            outcome
        });
    }

    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(ChapterOutcome::Persisted) => {},
            Ok(ChapterOutcome::Skipped { .. }) => skipped += 1,
            // Só cai aqui se a própria task deu panic (bug real, não erro de rede/protocolo) —
            // nenhum branch de `receive_one_chapter` propaga erro, só retorna o enum. Conta como
            // pulado em vez de abortar o resto da sessão por causa de UM item.
            Err(join_err) => {
                tracing::error!(error = %join_err, "[FileSync] chapter fetch task panicked — skipping");
                skipped += 1;
            },
        }
    }

    Ok(skipped)
}

/// Envia, em sequência, os itens extra (capa/banner/`ComicInfo.xml`) de `wanted_extras` — mesma
/// lógica de `send_files`, só que via `FileSyncService::resolve_local_extra`/`FileExtraHeader`
/// em vez de capítulo.
pub async fn send_extras(
    writer: &mut FramedWriter, wanted_extras: &[(String, String)], service: &FileSyncService,
    emit: &EventEmitter, progress_event: &str, transfer: &Arc<dyn ChapterTransfer>,
) -> Result<usize, P2pError> {
    let mut skipped = 0usize;

    for (comic_name, kind) in wanted_extras {
        let resolved = service.resolve_local_extra(comic_name, kind).await?;

        let Some((path, size, checksum, file_name)) = resolved else {
            tracing::warn!(
                comic_name = %comic_name,
                kind = %kind,
                "[FileSync] requested extra no longer exists locally — sending unavailable header"
            );
            skipped += 1;
            write_json(
                writer,
                &FileExtraHeader {
                    comic_name: comic_name.clone(),
                    kind: kind.clone(),
                    file_name: String::new(),
                    size: 0,
                    checksum: None,
                    blob_hash: None,
                },
            )
            .await?;
            continue;
        };

        let bytes = tokio::fs::read(&path).await.map_err(RpcError::from)?;
        let blob_hash = transfer.publish(bytes).await?;

        write_json(
            writer,
            &FileExtraHeader {
                comic_name: comic_name.clone(),
                kind: kind.clone(),
                file_name,
                size,
                checksum,
                blob_hash: Some(blob_hash),
            },
        )
        .await?;

        (emit)(progress_event, format!("{} - {}", comic_name, kind));
    }

    Ok(skipped)
}

/// Recebe, em sequência, `expected_count` itens extra: mesma mecânica de `receive_files`
/// (busca via blob store, grava em arquivo temporário, verifica checksum), mas persistindo via
/// `FileSyncService::persist_received_extra` em vez de `persist_received_chapter` — não vai pra
/// `chapter_archive`, vai pra `comic_directory` (capa/banner) ou vira `ComicInfo.xml` na raiz.
///
/// Ao terminar de persistir um `ComicInfo.xml`, dispara `MetadataService::sync_comic_comic_info`
/// (mesmo método usado pelo botão manual "sincronizar via ComicInfo") pra já deixar o
/// título/sinopse/autor atualizados no banco sem ação do usuário — melhor esforço: uma falha
/// aqui loga um aviso e conta como item não sincronizado nas estatísticas, mas não derruba a
/// sessão (o arquivo em si já foi transferido com sucesso).
#[allow(clippy::too_many_arguments)]
pub async fn receive_extras(
    reader: &mut FramedReader, expected_count: usize, service: &FileSyncService,
    metadata_service: &Arc<MetadataService>, emit: &EventEmitter, progress_event: &str,
    error_event: &str, peer: &PeerIdentity, transfer: &Arc<dyn ChapterTransfer>,
) -> Result<usize, P2pError> {
    let mut skipped = 0usize;

    for _ in 0..expected_count {
        let header: FileExtraHeader = read_json(reader).await?;

        if header.size == 0 {
            tracing::warn!(
                comic_name = %header.comic_name,
                kind = %header.kind,
                "[FileSync] peer reported extra as unavailable (size=0) — skipping"
            );
            skipped += 1;
            continue;
        }

        let Some(blob_hash) = header.blob_hash.as_deref() else {
            tracing::warn!(
                comic_name = %header.comic_name,
                kind = %header.kind,
                "[FileSync] extra header missing blob_hash — skipping"
            );
            skipped += 1;
            (emit)(
                error_event,
                item_error_payload(
                    &peer.id,
                    "missing blob hash",
                    Some(SyncErrorCode::MissingBlobHash),
                    &header.comic_name,
                    &header.kind,
                ),
            );
            continue;
        };

        let mut blob_reader = match transfer.fetch_reader(blob_hash, peer).await {
            Ok(reader) => reader,
            Err(err) => {
                tracing::warn!(
                    comic_name = %header.comic_name,
                    kind = %header.kind,
                    error = %err,
                    "[FileSync] failed to fetch extra blob — skipping"
                );
                skipped += 1;
                (emit)(
                    error_event,
                    item_error_payload(
                        &peer.id,
                        &format!("blob fetch failed: {err}"),
                        Some(classify_sync_error(&err).unwrap_or(SyncErrorCode::BlobFetchFailed)),
                        &header.comic_name,
                        &header.kind,
                    ),
                );
                continue;
            },
        };

        let mut bytes = Vec::new();
        blob_reader
            .read_to_end(&mut bytes)
            .await
            .map_err(|err| P2pError::StreamFailed(err.to_string()))?;

        let computed_checksum = {
            use sha2::{Digest, Sha256};
            format!("{:x}", Sha256::digest(&bytes))
        };

        if let Some(expected) = &header.checksum {
            if expected != &computed_checksum {
                tracing::warn!(
                    comic_name = %header.comic_name,
                    kind = %header.kind,
                    expected = %expected,
                    computed = %computed_checksum,
                    "[FileSync] extra checksum mismatch — skipping"
                );
                skipped += 1;
                (emit)(
                    error_event,
                    item_error_payload(
                        &peer.id,
                        "checksum mismatch",
                        Some(SyncErrorCode::ChecksumMismatch),
                        &header.comic_name,
                        &header.kind,
                    ),
                );
                continue;
            }
        }

        let comic_dir = match service.resolve_comic_dir(&header.comic_name).await {
            Ok(comic) => std::path::PathBuf::from(comic.path),
            Err(err) => {
                tracing::warn!(
                    comic_name = %header.comic_name,
                    kind = %header.kind,
                    error = %err,
                    "[FileSync] failed to resolve comic directory for extra — skipping"
                );
                skipped += 1;
                (emit)(
                    error_event,
                    item_error_payload(
                        &peer.id,
                        "failed to resolve comic directory",
                        Some(SyncErrorCode::ComicDirectoryUnavailable),
                        &header.comic_name,
                        &header.kind,
                    ),
                );
                continue;
            },
        };
        let temp_path = comic_dir.join(format!(".incoming-{}.tmp", rand::random::<u64>()));
        tokio::fs::write(&temp_path, &bytes).await.map_err(RpcError::from)?;

        let persisted = service
            .persist_received_extra(&header.comic_name, &header.kind, &header.file_name, &temp_path)
            .await;

        let comic = match persisted {
            Ok(comic) => comic,
            Err(err) => {
                tracing::warn!(
                    comic_name = %header.comic_name,
                    kind = %header.kind,
                    error = %err,
                    "[FileSync] failed to persist received extra — skipping"
                );
                skipped += 1;
                (emit)(
                    error_event,
                    item_error_payload(
                        &peer.id,
                        &format!("failed to persist extra: {err}"),
                        Some(SyncErrorCode::PersistFailed),
                        &header.comic_name,
                        &header.kind,
                    ),
                );
                continue;
            },
        };

        if header.kind == EXTRA_KIND_COMIC_INFO {
            if let Err(err) = metadata_service.sync_comic_comic_info(comic.id).await {
                tracing::warn!(
                    comic_name = %header.comic_name,
                    error = %err,
                    "[FileSync] ComicInfo.xml recebido, mas reprocessamento de metadata falhou"
                );
            }
        }

        (emit)(progress_event, format!("{} - {}", header.comic_name, header.kind));
    }

    Ok(skipped)
}

/// Duplo em memória de `ChapterTransfer` pra testes que não têm um `Endpoint` iroh de verdade
/// (`BlobChapterTransfer`, usado em produção, precisa disso). `shared_pair()` devolve dois
/// handles apontando pro mesmo `HashMap` interno — simula os dois lados publicando/buscando no
/// mesmo "store de rede" content-addressed.
#[cfg(test)]
pub struct InMemoryChapterTransfer {
    blobs: std::sync::Mutex<std::collections::HashMap<String, Vec<u8>>>,
    /// `None` por padrão (mesmo sentido de "sem conexão real pra medir" da doc do trait) —
    /// testes que exercitam o dimensionamento de paralelismo por latência usam
    /// `set_latency` pra simular um RTT específico sem precisar de rede de verdade.
    latency: std::sync::Mutex<Option<std::time::Duration>>,
}

#[cfg(test)]
impl InMemoryChapterTransfer {
    pub fn shared_pair() -> (Arc<dyn ChapterTransfer>, Arc<dyn ChapterTransfer>) {
        let shared = Arc::new(InMemoryChapterTransfer {
            blobs: std::sync::Mutex::new(std::collections::HashMap::new()),
            latency: std::sync::Mutex::new(None),
        });
        (Arc::clone(&shared) as Arc<dyn ChapterTransfer>, shared as Arc<dyn ChapterTransfer>)
    }

    pub fn set_latency(&self, latency: std::time::Duration) {
        *self.latency.lock().unwrap() = Some(latency);
    }
}

#[cfg(test)]
#[async_trait]
impl ChapterTransfer for InMemoryChapterTransfer {
    async fn publish(&self, bytes: Vec<u8>) -> Result<String, P2pError> {
        use sha2::{Digest, Sha256};
        let hash = format!("{:x}", Sha256::digest(&bytes));
        self.blobs.lock().unwrap().insert(hash.clone(), bytes);
        Ok(hash)
    }

    async fn fetch_reader(
        &self, blob_hash: &str, _peer: &PeerIdentity,
    ) -> Result<Box<dyn AsyncRead + Send + Unpin>, P2pError> {
        let bytes = self.blobs.lock().unwrap().get(blob_hash).cloned().ok_or_else(|| {
            P2pError::StreamFailed(format!("unknown blob hash in test transfer: {blob_hash}"))
        })?;

        // `tokio::io::duplex` já implementa `AsyncRead`/`AsyncWrite` — mais simples que
        // implementar um `poll_read` manual só pra teste. Escreve tudo e fecha (EOF) antes de
        // devolver a ponta de leitura.
        use tokio::io::AsyncWriteExt;
        let (mut writer, reader) = tokio::io::duplex(bytes.len().max(1));
        tokio::spawn(async move {
            let _ = writer.write_all(&bytes).await;
            let _ = writer.shutdown().await;
        });
        Ok(Box::new(reader))
    }

    async fn latency(&self, _peer: &PeerIdentity) -> Option<std::time::Duration> {
        *self.latency.lock().unwrap()
    }
}

#[cfg(test)]
mod concurrency_tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn chapter_fetch_concurrency_scales_with_measured_latency() {
        assert_eq!(chapter_fetch_concurrency(None), 2, "sem medição, fica no mais conservador");
        assert_eq!(chapter_fetch_concurrency(Some(Duration::from_millis(0))), 2);
        assert_eq!(
            chapter_fetch_concurrency(Some(Duration::from_millis(20))),
            2,
            "limite exato do degrau baixo, ainda dentro dele"
        );
        assert_eq!(
            chapter_fetch_concurrency(Some(Duration::from_millis(21))),
            4,
            "1ms acima do degrau baixo já sobe pro médio"
        );
        assert_eq!(
            chapter_fetch_concurrency(Some(Duration::from_millis(200))),
            4,
            "limite exato do degrau médio, ainda dentro dele"
        );
        assert_eq!(
            chapter_fetch_concurrency(Some(Duration::from_millis(201))),
            6,
            "1ms acima do degrau médio já sobe pro teto"
        );
        assert_eq!(
            chapter_fetch_concurrency(Some(Duration::from_secs(2))),
            6,
            "RTT bem alto (relay ruim) nunca passa do teto de 6"
        );
    }

    #[test]
    fn permits_to_forget_halves_the_pool_but_never_below_one_permit_left() {
        assert_eq!(permits_to_forget(6), 3, "6 -> mantém 3, esquece 3");
        assert_eq!(permits_to_forget(4), 2, "4 -> mantém 2, esquece 2");
        assert_eq!(permits_to_forget(2), 1, "2 -> mantém max(1,1)=1, esquece 1");
        assert_eq!(permits_to_forget(1), 0, "1 -> mantém max(1,0)=1, não há o que esquecer");
        assert_eq!(
            permits_to_forget(0),
            0,
            "0 -> nunca deveria acontecer na prática, mas não deve estourar (saturating_sub)"
        );
    }
}
