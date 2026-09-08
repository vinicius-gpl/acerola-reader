package br.acerola.comic.local.dao.sync

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.Query
import br.acerola.comic.local.entity.sync.SyncHistoryLog

@Dao
interface SyncHistoryLogDao {
    @Insert
    suspend fun insert(entry: SyncHistoryLog)

    /** Mais recentes primeiro — usado pra popular o log de atividade ao abrir a tela de sync. */
    @Query("SELECT * FROM sync_history_log ORDER BY created_at DESC, id DESC LIMIT :limit")
    suspend fun findRecent(limit: Int): List<SyncHistoryLog>

    /** Botão "Limpar" da tela de Rede — mesma operação do Desktop (`SyncHistoryLogRepository::delete_all`). */
    @Query("DELETE FROM sync_history_log")
    suspend fun deleteAll()
}
