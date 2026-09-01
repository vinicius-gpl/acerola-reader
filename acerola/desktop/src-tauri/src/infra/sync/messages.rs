use serde::{Deserialize, Serialize};

/// Uma entrada de progresso de leitura, identificada pela chave natural
/// (nome do quadrinho + rótulo do capítulo) — a única forma comparável entre
/// duas bases SQLite independentes, já que os IDs autoincrement não são portáveis.
///
/// Nomes de campo no wire (`chapter_sort` em vez de `chapter`) seguem o schema já
/// usado pelo Android em `protocol/history/model.rs` — schema compartilhado do ALPN
/// `acerola/sync-history/1`, os nomes Rust internos não mudam.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub comic_name: String,
    #[serde(rename = "chapter_sort")]
    pub chapter: String,
    pub last_page: i64,
    pub is_completed: bool,
    pub updated_at: i64,
}

/// Um marcador de "capítulo lido". Não tem `updated_at` — é um fato binário que só
/// existe ou não, então o merge entre devices é uma união (insert_or_ignore), não
/// last-write-wins.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadMarker {
    pub comic_name: String,
    #[serde(rename = "chapter_sort")]
    pub chapter: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HistoryManifest {
    #[serde(rename = "reading_progress")]
    pub entries: Vec<HistoryEntry>,
    #[serde(rename = "chapters_read")]
    pub read_markers: Vec<ReadMarker>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChapterInfo {
    pub chapter: String,
    pub file_name: String,
    pub checksum: Option<String>,
    pub size: u64,
}

/// Metadado de um item "extra" por quadrinho (capa, banner ou `ComicInfo.xml`) — mesmo shape
/// pra os três, distinguidos pelo campo que os carrega em `FileComicInfo` (`cover`/`banner`/
/// `comic_info`) e, na hora da transferência, pelo campo `kind` de `FileExtraHeader`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileExtraInfo {
    pub file_name: String,
    /// SHA-256 hex dos bytes do arquivo — recalculado a cada `build_manifest()` (arquivos
    /// pequenos, ao contrário dos capítulos, então não precisa de um checksum cacheado em
    /// banco como `ChapterArchive.checksum`).
    pub checksum: String,
    pub size: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileComicInfo {
    pub comic_name: String,
    pub chapters: Vec<FileChapterInfo>,
    /// `None` cobre tanto "quadrinho sem capa salva" quanto "arquivo da capa sumiu do disco" —
    /// os três campos abaixo (`cover`/`banner`/`comic_info`) seguem essa mesma convenção.
    /// `skip_serializing_if` omite a chave em vez de escrever `null` quando ausente — um
    /// quadrinho sem nenhum dos três produz exatamente o mesmo JSON de antes desta mudança.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover: Option<FileExtraInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banner: Option<FileExtraInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comic_info: Option<FileExtraInfo>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileManifest {
    pub comics: Vec<FileComicInfo>,
}

pub const EXTRA_KIND_COVER: &str = "cover";
pub const EXTRA_KIND_BANNER: &str = "banner";
pub const EXTRA_KIND_COMIC_INFO: &str = "comic_info";

/// O que este lado quer *do outro lado* — calculado localmente depois de comparar
/// os dois manifestos, e enviado de volta pro peer decidir o que transmitir.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileWantList {
    pub wanted: Vec<(String, String)>,
    /// Pares `(comic_name, kind)`, `kind` sendo um dos `EXTRA_KIND_*` — mesma ideia de
    /// `wanted`, só que pra capa/banner/ComicInfo.xml em vez de capítulo. `#[serde(default)]`
    /// pra um peer sem essa versão do protocolo ainda desserializar como lista vazia em vez de
    /// falhar; `skip_serializing_if` mantém o wire idêntico a antes desta mudança quando não
    /// há nenhum item extra pra pedir.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub wanted_extras: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHeader {
    pub comic_name: String,
    pub chapter: String,
    pub file_name: String,
    pub size: u64,
    /// SHA-256 do manifesto (`FileChapterInfo.checksum`) — repassado tal e qual pra
    /// verificação local de escrita, sem relação com blobs.
    pub checksum: Option<String>,
    /// Hash de blob (BLAKE3, hex) devolvido por `P2pBlobStore::put` no lado que enviou —
    /// é o que o lado que recebe usa em `P2pBlobStore::fetch` pra puxar os bytes de verdade.
    /// `None` só no header vazio de "não tenho mais esse capítulo" (`size: 0`).
    pub blob_hash: Option<String>,
}

/// Header de transferência de um item extra (capa/banner/`ComicInfo.xml`) — paralelo a
/// `FileHeader`, mas com `kind` (`EXTRA_KIND_*`) no lugar de `chapter`: são conceitos
/// diferentes (um item extra não pertence a nenhum capítulo), um header dedicado evita
/// sobrecarregar o campo `chapter` com um valor que não é um capítulo de verdade.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileExtraHeader {
    pub comic_name: String,
    pub kind: String,
    pub file_name: String,
    pub size: u64,
    /// `None` só no header vazio de "não tenho mais esse item" (`size: 0`) — mesma convenção
    /// de `FileHeader`.
    pub checksum: Option<String>,
    pub blob_hash: Option<String>,
}

/// Primeira mensagem do protocolo `acerola/sync-comic/1`, escrita pelo lado que inicia — diz
/// ao peer qual quadrinho escopar antes da troca de manifestos (ver
/// `FileSyncService::build_manifest_for_comic`). Sem isso o lado que responde não teria como
/// saber qual quadrinho escopar quando o iniciador ainda não tem nenhum capítulo dele
/// localmente (caso "pull": o manifesto do iniciador viria vazio, sem nome nenhum dentro).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComicSyncRequest {
    pub comic_name: String,
}

/// Marcador mínimo escrito pelo lado outbound de `acerola/browse-library/1` antes de ler a
/// resposta — não carrega nenhum dado de verdade (o pedido em si é "liste sua biblioteca"),
/// mas *precisa* existir por causa de uma regra da própria `quinn` (biblioteca QUIC por baixo
/// do `iroh`), documentada em `Connection::open_bi()`: "the Connection that calls open_bi()
/// must write to its SendStream before the other Connection is able to accept_bi() (...)
/// waiting on the RecvStream without writing anything to SendStream will never succeed".
///
/// Era exatamente esse o bug reaberto em 22/08/2026 (timeout de 30s, `FRAME_TIMEOUT`, nos dois
/// lados): `LibraryBrowseOutbound` abria o stream e só lia, nunca escrevia nada — o
/// `accept_bi()` do lado inbound nunca disparava, então o handler inbound nunca era sequer
/// invocado, e o outbound ficava esperando uma resposta que nunca seria escrita. Os outros
/// protocolos (`sync-history`, `sync-files`, `sync-comic`) nunca tiveram esse problema porque o
/// lado outbound deles sempre escreve algo primeiro (`FileManifest`/`ComicSyncRequest`).
///
/// O lado inbound só precisa drenar essa mensagem, não olhar o conteúdo dela — por isso não
/// carrega nenhum campo.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LibraryBrowseRequest {}

/// Resumo de um quadrinho da biblioteca remota, usado só pra listar/buscar (sem transferir
/// nada ainda) — ver protocolo `acerola/browse-library/1`. Nomes espelhados no Android
/// (`protocol/library_browse/model.rs::ComicSummaryEntry`/`LibrarySummary`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComicSummaryEntry {
    pub comic_name: String,
    pub chapter_count: u32,
    /// Reaproveita `ComicDirectory.last_modified`/`comic_directory.last_modified` — sem hash
    /// novo. O cliente compara contra a versão já cacheada localmente pra decidir se precisa
    /// buscar uma capa nova via `acerola/browse-cover/1`.
    pub cover_version: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LibrarySummary {
    pub comics: Vec<ComicSummaryEntry>,
}

/// Único frame escrito pelo lado outbound de `acerola/browse-cover/1` — a conexão em si não é
/// o pedido (diferente de `browse-library`), porque aqui o outbound precisa informar QUAL
/// quadrinho e qual versão já tem cacheada, pra o inbound decidir entre `not_modified` e
/// publicar um blob novo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverRequest {
    pub comic_name: String,
    /// `None` = "nunca busquei essa capa antes" — qualquer versão que o inbound tiver conta
    /// como "mudou".
    pub known_version: Option<i64>,
}

pub const COVER_STATUS_NOT_MODIFIED: &str = "not_modified";
pub const COVER_STATUS_CHANGED: &str = "changed";
pub const COVER_STATUS_UNAVAILABLE: &str = "unavailable";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverResponse {
    pub status: String,
    pub cover_version: Option<i64>,
    /// Só presente quando `status == "changed"` — hash de blob (BLAKE3, hex) que o outbound usa
    /// em `ChapterTransfer::fetch_reader` pra puxar os bytes de verdade.
    pub cover_hash: Option<String>,
}

/// Mensagem enviada no lugar do manifesto quando `FileSyncSessionGuard` já rejeitou a sessão
/// localmente (peer já tem uma sessão ativa) — schema espelhado no Android
/// (`protocol/files/model.rs::SessionBusy`), único jeito dos dois lados falarem a mesma coisa
/// mesmo sem compartilhar código. O campo `error` é o discriminador: nenhuma outra mensagem
/// do protocolo de arquivos usa essa chave (`FileManifest` usa `comics`, `FileWantList` usa
/// `wanted`), então quem lê pode detectar isso antes de tentar desserializar como
/// `FileManifest`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionBusy {
    pub error: String,
    pub reason: String,
}

#[cfg(test)]
mod wire_contract_tests {
    use super::*;

    /// Trava o schema de wire do `HistoryManifest` contra o formato que o Android
    /// (`protocol/history/model.rs` + `callbacks.rs`) produz e espera — os dois lados
    /// não compartilham código, então esse teste é o que garante que não divergem de
    /// novo silenciosamente.
    #[test]
    fn history_manifest_matches_android_wire_shape() {
        let manifest = HistoryManifest {
            entries: vec![HistoryEntry {
                comic_name: "Berserk".into(),
                chapter: "12".into(),
                last_page: 5,
                is_completed: false,
                updated_at: 1000,
            }],
            read_markers: vec![ReadMarker {
                comic_name: "Berserk".into(),
                chapter: "11".into(),
                created_at: 900,
            }],
        };

        let value = serde_json::to_value(&manifest).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "reading_progress": [{
                    "comic_name": "Berserk",
                    "chapter_sort": "12",
                    "last_page": 5,
                    "is_completed": false,
                    "updated_at": 1000
                }],
                "chapters_read": [{
                    "comic_name": "Berserk",
                    "chapter_sort": "11",
                    "created_at": 900
                }]
            })
        );

        // Mensagem exatamente no formato que o Android envia (chaves reading_progress/
        // chapters_read, campo chapter_sort) precisa desserializar sem erro.
        let android_wire = serde_json::json!({
            "reading_progress": [{
                "comic_name": "Berserk",
                "chapter_sort": "12",
                "last_page": 5,
                "is_completed": false,
                "updated_at": 1000
            }],
            "chapters_read": []
        });
        let decoded: HistoryManifest = serde_json::from_value(android_wire).unwrap();
        assert_eq!(decoded.entries[0].chapter, "12");
    }

    /// `LibraryBrowseRequest` só precisa existir no wire (ver doc do tipo) — trava que o
    /// formato é um objeto vazio, então um peer antigo (design "conexão é o pedido", sem
    /// nenhuma leitura antes de responder) recebe um JSON inofensivo e ignorável, não algo
    /// que possa confundir um parser mais estrito do outro lado.
    #[test]
    fn library_browse_request_serializes_as_empty_object() {
        let value = serde_json::to_value(LibraryBrowseRequest::default()).unwrap();
        assert_eq!(value, serde_json::json!({}));
    }

    /// Trava o schema de wire de `FileComicInfo` com os campos `cover`/`banner`/`comic_info`
    /// novos contra o formato produzido/esperado pelo Android
    /// (`protocol/files/model.rs::FileComicInfo`) — mesma convenção dos outros testes desta
    /// suíte: sem tag de enum, `#[serde(default)]` cobrindo o caso "item ausente".
    #[test]
    fn file_comic_info_matches_android_wire_shape_with_extras() {
        let comic = FileComicInfo {
            comic_name: "Berserk".into(),
            chapters: vec![FileChapterInfo {
                chapter: "Cap 1".into(),
                file_name: "Cap 1.cbz".into(),
                checksum: Some("abc123".into()),
                size: 42,
            }],
            cover: Some(FileExtraInfo {
                file_name: "cover.jpg".into(),
                checksum: "cover-hash".into(),
                size: 1024,
            }),
            banner: None,
            comic_info: Some(FileExtraInfo {
                file_name: "ComicInfo.xml".into(),
                checksum: "info-hash".into(),
                size: 256,
            }),
        };

        let value = serde_json::to_value(&comic).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "comic_name": "Berserk",
                "chapters": [{
                    "chapter": "Cap 1",
                    "file_name": "Cap 1.cbz",
                    "checksum": "abc123",
                    "size": 42
                }],
                "cover": { "file_name": "cover.jpg", "checksum": "cover-hash", "size": 1024 },
                "comic_info": { "file_name": "ComicInfo.xml", "checksum": "info-hash", "size": 256 }
            })
        );

        // Payload de um peer antigo, sem os campos novos — precisa desserializar como None,
        // não falhar.
        let old_peer_wire = serde_json::json!({
            "comic_name": "Berserk",
            "chapters": []
        });
        let decoded: FileComicInfo = serde_json::from_value(old_peer_wire).unwrap();
        assert!(decoded.cover.is_none());
        assert!(decoded.banner.is_none());
        assert!(decoded.comic_info.is_none());
    }

    /// Trava o schema de wire de `FileWantList.wanted_extras` — mesmo padrão de par de
    /// strings usado por `wanted`, só que `(comic_name, kind)` em vez de `(comic_name,
    /// chapter)`.
    #[test]
    fn file_want_list_serializes_wanted_extras_as_tuple_array() {
        let wanted = FileWantList {
            wanted: vec![("Berserk".into(), "Cap 1".into())],
            wanted_extras: vec![("Berserk".into(), EXTRA_KIND_COVER.into())],
        };

        let value = serde_json::to_value(&wanted).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "wanted": [["Berserk", "Cap 1"]],
                "wanted_extras": [["Berserk", "cover"]]
            })
        );

        // Peer antigo sem `wanted_extras` no payload — desserializa como lista vazia.
        let old_peer_wire = serde_json::json!({ "wanted": [] });
        let decoded: FileWantList = serde_json::from_value(old_peer_wire).unwrap();
        assert!(decoded.wanted_extras.is_empty());
    }

    /// Trava o schema de wire de `FileExtraHeader` contra o formato espelhado no Android
    /// (`protocol/files/model.rs::FileExtraHeader`).
    #[test]
    fn file_extra_header_matches_android_wire_shape() {
        let header = FileExtraHeader {
            comic_name: "Berserk".into(),
            kind: EXTRA_KIND_BANNER.into(),
            file_name: "banner.png".into(),
            size: 2048,
            checksum: Some("banner-hash".into()),
            blob_hash: Some("blake3-hex".into()),
        };

        let value = serde_json::to_value(&header).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "comic_name": "Berserk",
                "kind": "banner",
                "file_name": "banner.png",
                "size": 2048,
                "checksum": "banner-hash",
                "blob_hash": "blake3-hex"
            })
        );
    }
}
