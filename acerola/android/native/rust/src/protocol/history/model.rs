use serde::{Deserialize, Serialize};

use crate::callbacks::{FfiChapterReadEntry, FfiReadingProgressEntry};

/// Manifesto trocado uma única vez por sessão do protocolo `acerola/sync-history/1` —
/// contém o estado local inteiro de progresso de leitura e capítulos lidos, já que ambos
/// são pequenos (sem payload binário), permitindo que cada lado aplique sua própria lógica
/// de diff/conflito localmente, sem uma segunda rodada de rede.
///
/// Schema de wire compartilhado com o Desktop (`acerola-desktop/.../infra/sync/messages.rs`,
/// via `#[serde(rename)]` nos campos `entries`/`read_markers`/`chapter`). Os dois lados não
/// compartilham código, então os nomes de campo aqui (`reading_progress`, `chapters_read`,
/// `chapter_sort`) são o contrato — não renomear sem atualizar o outro lado também.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct HistoryManifest {
    pub reading_progress: Vec<FfiReadingProgressEntry>,
    pub chapters_read: Vec<FfiChapterReadEntry>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct HistorySyncStats {
    pub progress_applied: u32,
    pub progress_skipped: u32,
    pub chapters_read_applied: u32,
    pub chapters_read_skipped: u32,
}

/// Primeira mensagem do protocolo `acerola/sync-history-entry/1`, escrita pelo lado outbound —
/// declara explicitamente qual quadrinho e quais capítulos (`chapter_sort`) o manifesto que vem
/// a seguir está escopado, em vez de o inbound só descobrir isso lendo o conteúdo do manifesto
/// (que pode vir vazio se nenhum capítulo selecionado tiver progresso/marcador de "lido" — nesse
/// caso o inbound não teria como saber nem qual quadrinho validar). Schema espelhado no Desktop
/// (`infra/sync/messages.rs::HistoryEntryRequest`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoryEntryRequest {
    pub comic_name: String,
    pub chapter_sorts: Vec<String>,
}

/// Resposta do lado inbound de `acerola/sync-history-entry/1`, no lugar de um ack vazio —
/// carrega o resultado real da aplicação. `comic_known` é o que permite o outbound diferenciar
/// "enviei e o peer aplicou" de "enviei, mas o peer nem tinha esse quadrinho" (antes um falso
/// positivo silencioso). Schema espelhado no Desktop
/// (`infra/sync/messages.rs::HistoryEntryAck`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoryEntryAck {
    pub comic_known: bool,
    pub entries_applied: u32,
    pub markers_applied: u32,
}
