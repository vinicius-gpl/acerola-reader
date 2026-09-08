package br.acerola.comic.usecase.network

import br.acerola.comic.local.dao.sync.SyncHistoryLogDao
import br.acerola.comic.local.entity.sync.SyncHistoryLog
import javax.inject.Inject

/**
 * Persisted history of P2P sync sessions (history and files) — terminal states only
 * (`complete`/`error`). Mirrors `sync_history_log` on Desktop
 * (`infra/db/migrations/models/sync/001_create_sync_history_log.sql`); exists to survive an
 * app restart, complementing [br.acerola.comic.service.network.P2pEventBus] (which only
 * covers the current session).
 */
class SyncHistoryLogUseCase
    @Inject
    constructor(
        private val dao: SyncHistoryLogDao,
    ) {
        suspend fun record(
            peerId: String,
            kind: String,
            status: String,
            message: String?,
        ) {
            dao.insert(SyncHistoryLog(peerId = peerId, kind = kind, status = status, message = message))
        }

        suspend fun findRecent(limit: Int): List<SyncHistoryLog> = dao.findRecent(limit)

        suspend fun clearAll() = dao.deleteAll()
    }
