use sqlx::SqlitePool;

use crate::{
    data::{models::sync::SyncHistoryLogEntry, repositories::Repository},
    infra::error::DbError,
};

/// Histórico persistido de sessões de sync P2P — só estados terminais (`complete`/`error`),
/// nunca `started`/`progress`. `kind` é `"history"` ou `"files"`, `status` é `"complete"`
/// ou `"error"`. Segue o mesmo padrão `Entity`/`Bindable`/`Repository<T>` de todo model
/// desta pasta (ver `SyncHistoryLogEntry`); `find_recent` é custom porque o genérico
/// `Repository::find_all` não suporta ORDER BY/LIMIT.
#[derive(Clone)]
pub struct SyncHistoryLogRepository {
    pub base: Repository<SyncHistoryLogEntry>,
    pool: SqlitePool,
}

impl SyncHistoryLogRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { base: Repository::new(pool.clone()), pool }
    }

    /// Últimas `limit` linhas, mais recente primeiro — usado pra popular o log de
    /// atividade ao abrir a tela de rede e pra calcular "última sincronização" por peer.
    pub async fn find_recent(&self, limit: i64) -> Result<Vec<SyncHistoryLogEntry>, DbError> {
        let rows = sqlx::query_as::<_, SyncHistoryLogEntry>(
            "SELECT id, peer_id, kind, status, message, created_at
             FROM sync_history_log
             ORDER BY created_at DESC, id DESC
             LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::SyncHistoryLogRepository;
    use crate::{
        data::models::sync::SyncHistoryLogEntry,
        tests::utils::setup_test_db::setup_test_db_with_comic,
    };

    #[tokio::test]
    async fn insert_and_find_recent_orders_newest_to_oldest() {
        let pool = setup_test_db_with_comic().await;
        let repo = SyncHistoryLogRepository::new(pool);

        repo.base
            .insert(&SyncHistoryLogEntry::new("peer-a", "history", "complete", None))
            .await
            .unwrap();
        repo.base
            .insert(&SyncHistoryLogEntry::new("peer-a", "files", "error", Some("boom")))
            .await
            .unwrap();

        let recent = repo.find_recent(10).await.unwrap();
        assert_eq!(recent.len(), 2);
        // A segunda inserção (files/error) deve vir primeiro (mais recente).
        assert_eq!(recent[0].kind, "files");
        assert_eq!(recent[0].status, "error");
        assert_eq!(recent[0].message.as_deref(), Some("boom"));
        assert_eq!(recent[1].kind, "history");
        assert_eq!(recent[1].message, None);
    }

    #[tokio::test]
    async fn find_recent_respects_the_limit() {
        let pool = setup_test_db_with_comic().await;
        let repo = SyncHistoryLogRepository::new(pool);

        for _ in 0..5 {
            repo.base
                .insert(&SyncHistoryLogEntry::new("peer-a", "history", "complete", None))
                .await
                .unwrap();
        }

        let recent = repo.find_recent(3).await.unwrap();
        assert_eq!(recent.len(), 3);
    }
}
