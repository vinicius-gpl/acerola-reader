use serde::{Deserialize, Serialize};

use crate::callbacks::{FfiExtraManifestEntry, FfiFileManifestEntry};

/// Todos os tipos de mensagem trocados ao longo de uma sessão do protocolo
/// `acerola/sync-files/1`. Schema compartilhado com o Desktop
/// (`acerola-desktop/.../infra/sync/messages.rs`) — os nomes e formatos aqui precisam
/// bater exatamente com os de lá, já que os dois lados não compartilham código.
///
/// Cada mensagem é um frame JSON solto (sem tag de enum) via length-delimited codec.
/// Os bytes de um capítulo não viajam dentro de uma mensagem JSON — são frames binários
/// crus enviados logo após o `FileHeader` correspondente, até somar `size` bytes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FileChapterInfo {
    pub chapter: String,
    pub file_name: String,
    pub checksum: Option<String>,
    pub size: u64,
}

/// Metadado de um item "extra" por quadrinho (capa, banner ou `ComicInfo.xml`) — schema
/// espelhado no Desktop (`infra/sync/messages.rs::FileExtraInfo`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FileExtraInfo {
    pub file_name: String,
    pub checksum: String,
    pub size: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct FileComicInfo {
    pub comic_name: String,
    pub chapters: Vec<FileChapterInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover: Option<FileExtraInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banner: Option<FileExtraInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comic_info: Option<FileExtraInfo>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct FileManifest {
    pub comics: Vec<FileComicInfo>,
}

pub(crate) const EXTRA_KIND_COVER: &str = "cover";
pub(crate) const EXTRA_KIND_BANNER: &str = "banner";
pub(crate) const EXTRA_KIND_COMIC_INFO: &str = "comic_info";

/// O que este lado quer *do outro lado* — pares `(comic_name, chapter)`, mais
/// `wanted_extras`: pares `(comic_name, kind)` pra capa/banner/ComicInfo.xml.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct FileWantList {
    pub wanted: Vec<(String, String)>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub wanted_extras: Vec<(String, String)>,
}

/// Fase 0 exclusiva do protocolo `acerola/sync-comic/1` (sincronização de um único
/// quadrinho) — trocada antes de qualquer coisa do protocolo `sync-files` normal. Só o lado
/// outbound sabe qual quadrinho o usuário escolheu (veio de uma chamada FFI Kotlin ->
/// `P2PNode::sync_comic`, não do wire); o inbound aprende por aqui e filtra seu próprio
/// manifesto ao mesmo `comic_name` antes de seguir para as fases 1-3 compartilhadas com
/// `run_exchange`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ComicSyncScope {
    pub comic_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FileHeader {
    pub comic_name: String,
    pub chapter: String,
    pub file_name: String,
    pub size: u64,
    /// SHA-256 do manifesto (`FileChapterInfo.checksum`) — repassado tal e qual pra
    /// `FileSyncProvider::begin_chapter_write`/verificação Kotlin, sem relação com blobs.
    pub checksum: Option<String>,
    /// Hash de blob (BLAKE3, hex) devolvido por `P2pBlobStore::put` no lado que enviou —
    /// é o que o lado que recebe usa em `P2pBlobStore::fetch` pra puxar os bytes de verdade.
    /// `None` só no header vazio de "não tenho mais esse capítulo" (`size: 0`).
    pub blob_hash: Option<String>,
}

/// Header de transferência de um item extra (capa/banner/`ComicInfo.xml`) — paralelo a
/// `FileHeader`, schema espelhado no Desktop (`infra/sync/messages.rs::FileExtraHeader`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FileExtraHeader {
    pub comic_name: String,
    pub kind: String,
    pub file_name: String,
    pub size: u64,
    pub checksum: Option<String>,
    pub blob_hash: Option<String>,
}

/// Mensagem enviada no lugar do manifesto quando o `FileSyncSessionGuard` já rejeitou a
/// sessão localmente (peer já tem uma sessão ativa) — schema espelhado no Desktop
/// (`file_handler.rs`), único jeito dos dois lados falarem a mesma coisa mesmo sem
/// compartilhar código. O campo `error` é o discriminador: nenhuma outra mensagem do
/// protocolo usa essa chave (`FileManifest` usa `comics`, `FileWantList` usa `wanted`), então
/// quem lê pode detectar isso antes de tentar desserializar como `FileManifest`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SessionBusy {
    pub error: String,
    pub reason: String,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct FileSyncStats {
    pub received_count: u32,
    pub sent_count: u32,
    pub failed_count: u32,
}

/// Agrupa a lista plana de `get_file_manifest()` (FFI) em `FileManifest` aninhado por
/// quadrinho — formato de wire compartilhado com o Desktop.
pub(super) fn build_manifest(entries: Vec<FfiFileManifestEntry>) -> FileManifest {
    use std::collections::HashMap;

    let mut by_comic: HashMap<String, Vec<FileChapterInfo>> = HashMap::new();
    for entry in entries {
        by_comic.entry(entry.comic_name).or_default().push(FileChapterInfo {
            chapter: entry.chapter,
            file_name: entry.file_name,
            checksum: Some(entry.checksum),
            size: entry.size_bytes,
        });
    }

    let comics = by_comic
        .into_iter()
        .map(|(comic_name, chapters)| FileComicInfo { comic_name, chapters, ..Default::default() })
        .collect();

    FileManifest { comics }
}

/// Funde os itens extra (capa/banner/`ComicInfo.xml`) de `get_extras_manifest()` (FFI) dentro
/// do `FileManifest` já montado por `build_manifest()` — cria uma entrada de comic nova (sem
/// capítulo nenhum) se o único dado que existe pra ele for um extra.
pub(super) fn merge_extras(manifest: &mut FileManifest, extras: Vec<FfiExtraManifestEntry>) {
    use std::collections::HashMap;

    // Índices (não `&mut FileComicInfo` direto) porque o `push` de um comic novo mais abaixo
    // pode realocar `manifest.comics` — qualquer referência emprestada antes do `push` ficaria
    // inválida. Chaves são `String` própria (não emprestada de `manifest`) exatamente pra poder
    // inserir uma entrada nova nesse mesmo mapa sem conflitar com o empréstimo de `manifest`.
    let mut index_by_comic: HashMap<String, usize> =
        manifest.comics.iter().enumerate().map(|(index, comic)| (comic.comic_name.clone(), index)).collect();

    for extra in extras {
        let info = FileExtraInfo { file_name: extra.file_name, checksum: extra.checksum, size: extra.size_bytes };

        let index = *index_by_comic.entry(extra.comic_name.clone()).or_insert_with(|| {
            manifest.comics.push(FileComicInfo { comic_name: extra.comic_name.clone(), ..Default::default() });
            manifest.comics.len() - 1
        });

        assign_extra(&mut manifest.comics[index], &extra.kind, info);
    }
}

fn assign_extra(comic: &mut FileComicInfo, kind: &str, info: FileExtraInfo) {
    match kind {
        EXTRA_KIND_COVER => comic.cover = Some(info),
        EXTRA_KIND_BANNER => comic.banner = Some(info),
        EXTRA_KIND_COMIC_INFO => comic.comic_info = Some(info),
        _ => {},
    }
}

#[cfg(test)]
mod wire_contract_tests {
    use super::*;

    /// Trava o schema de wire do `sync-files` contra o que o Desktop produz/espera
    /// (`acerola-desktop/.../infra/sync/messages.rs`) — os dois lados não compartilham
    /// código, então isso é o que garante que não divergem de novo silenciosamente.
    #[test]
    fn file_manifest_matches_desktop_wire_shape() {
        let manifest = build_manifest(vec![FfiFileManifestEntry {
            comic_name: "Berserk".into(),
            chapter: "Cap 1".into(),
            file_name: "Cap 1.cbz".into(),
            checksum: "abc123".into(),
            size_bytes: 42,
        }]);

        let value = serde_json::to_value(&manifest).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "comics": [{
                    "comic_name": "Berserk",
                    "chapters": [{
                        "chapter": "Cap 1",
                        "file_name": "Cap 1.cbz",
                        "checksum": "abc123",
                        "size": 42
                    }]
                }]
            })
        );

        // Payload no exato formato que o Desktop envia (chave "comics", sem tag de enum)
        // precisa desserializar sem erro.
        let desktop_wire = serde_json::json!({
            "comics": [{
                "comic_name": "Berserk",
                "chapters": [{
                    "chapter": "Cap 1",
                    "file_name": "Cap 1.cbz",
                    "checksum": "abc123",
                    "size": 42
                }]
            }]
        });
        let decoded: FileManifest = serde_json::from_value(desktop_wire).unwrap();
        assert_eq!(decoded.comics[0].chapters[0].file_name, "Cap 1.cbz");
    }

    #[test]
    fn file_want_list_serializes_as_tuple_array() {
        let wanted = FileWantList { wanted: vec![("Berserk".into(), "Cap 1".into())], ..Default::default() };
        let value = serde_json::to_value(&wanted).unwrap();
        assert_eq!(value, serde_json::json!({ "wanted": [["Berserk", "Cap 1"]] }));
    }

    /// Trava o schema de wire de `FileComicInfo` com os campos `cover`/`banner`/`comic_info`
    /// novos contra o formato espelhado no Desktop
    /// (`infra/sync/messages.rs::FileComicInfo`) — `skip_serializing_if` garante que um
    /// quadrinho sem nenhum item extra produz o mesmo JSON de antes desta mudança.
    #[test]
    fn file_comic_info_matches_desktop_wire_shape_with_extras() {
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

        // Payload de um peer antigo (Desktop sem essa versão do protocolo) — desserializa
        // como None, não falha.
        let old_peer_wire = serde_json::json!({ "comic_name": "Berserk", "chapters": [] });
        let decoded: FileComicInfo = serde_json::from_value(old_peer_wire).unwrap();
        assert!(decoded.cover.is_none());
        assert!(decoded.banner.is_none());
        assert!(decoded.comic_info.is_none());
    }

    /// Trava o schema de wire de `FileWantList.wanted_extras` contra o Desktop.
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

        let old_peer_wire = serde_json::json!({ "wanted": [] });
        let decoded: FileWantList = serde_json::from_value(old_peer_wire).unwrap();
        assert!(decoded.wanted_extras.is_empty());
    }

    /// Trava o schema de wire de `FileExtraHeader` contra o Desktop
    /// (`infra/sync/messages.rs::FileExtraHeader`).
    #[test]
    fn file_extra_header_matches_desktop_wire_shape() {
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
