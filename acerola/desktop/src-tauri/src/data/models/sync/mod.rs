use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sqlx::{query::Query, sqlite::SqliteArguments, Sqlite};

use crate::data::repositories::{Bindable, Entity};

/// Uma linha do histórico persistido de sessões de sync P2P (histórico e arquivos) —
/// só estados terminais (`complete`/`error`), não `started`/`progress`. Existe pra sobreviver
/// a um restart do app; o log "ao vivo" em memória continua existindo à parte pra eventos
/// que chegam em tempo real durante a sessão atual.
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SyncHistoryLogEntry {
    pub id: i64,
    pub peer_id: String,
    pub kind: String,
    pub status: String,
    pub message: Option<String>,
    pub created_at: i64,
}

/// Contrato com o [`crate::data::repositories::Repository`] genérico — mesmo padrão de
/// todo model nesta pasta (`ChapterArchive`, `ChapterRead`, etc), mesmo sendo uma tabela
/// append-only: `id` é calculado no Rust (não confia em AUTOINCREMENT do SQLite) e
/// vinculado explicitamente no insert, igual `ChapterArchive::id` usa `path_hash(...)`.
impl Entity for SyncHistoryLogEntry {
    fn columns() -> &'static [&'static str] {
        &["id", "peer_id", "kind", "status", "message", "created_at"]
    }
    fn table_name() -> &'static str {
        "sync_history_log"
    }
    fn id(&self) -> i64 {
        self.id
    }
}

impl Bindable for SyncHistoryLogEntry {
    fn bind_insert<'query>(
        &'query self, query: Query<'query, Sqlite, SqliteArguments<'query>>,
    ) -> Query<'query, Sqlite, SqliteArguments<'query>> {
        query
            .bind(self.id)
            .bind(&self.peer_id)
            .bind(&self.kind)
            .bind(&self.status)
            .bind(&self.message)
            .bind(self.created_at)
    }

    fn bind_update<'query>(
        &'query self, query: Query<'query, Sqlite, SqliteArguments<'query>>,
    ) -> Query<'query, Sqlite, SqliteArguments<'query>> {
        query
            .bind(&self.peer_id)
            .bind(&self.kind)
            .bind(&self.status)
            .bind(&self.message)
            .bind(self.created_at)
            .bind(self.id) // <- id pro WHERE id = ?
    }
}

impl SyncHistoryLogEntry {
    /// `id` calculado no Rust (epoch em nanossegundos) em vez de deixar o SQLite
    /// autoincrementar — mesma convenção de `ChapterArchive::id` (via `path_hash`), pro
    /// `Bindable::bind_insert` poder bindar o id explicitamente. Colisão exigiria duas
    /// chamadas no mesmo nanossegundo, que não acontece pra eventos de sync (rede é ordens
    /// de magnitude mais lenta que isso).
    pub fn new(peer_id: &str, kind: &str, status: &str, message: Option<&str>) -> Self {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();

        Self {
            id: now.as_nanos() as i64,
            peer_id: peer_id.to_string(),
            kind: kind.to_string(),
            status: status.to_string(),
            message: message.map(str::to_string),
            created_at: now.as_millis() as i64,
        }
    }
}
