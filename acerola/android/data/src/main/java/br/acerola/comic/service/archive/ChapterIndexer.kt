package br.acerola.comic.service.archive

import android.content.Context
import android.net.Uri
import androidx.documentfile.provider.DocumentFile
import br.acerola.comic.local.entity.archive.ChapterArchive
import br.acerola.comic.local.translator.persistence.toChapterArchiveEntity
import br.acerola.comic.logging.AcerolaLogger
import br.acerola.comic.logging.LogSource
import br.acerola.comic.util.file.FastFileMetadata
import br.acerola.comic.util.file.sha256
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class ChapterIndexer
    @Inject
    constructor(
        @param:ApplicationContext private val context: Context,
    ) {
        /**
         * Calcula o checksum aqui, no scan — em background, sem timeout de rede em cima —
         * em vez de deixar pra hora do sync P2P (`FileSyncProvider.getFileManifest`), que era
         * o pior momento possível pra ler o arquivo inteiro. `ChapterSyncService.sync()` só
         * chama isto pra capítulos novos ou com `lastModified` alterado; um capítulo já
         * indexado e sem mudança nunca passa por aqui de novo (ver o `return@forEachIndexed`
         * ali), então o custo é incremental a cada scan, não a biblioteca inteira toda vez.
         */
        suspend fun buildEntity(
            file: FastFileMetadata,
            comicId: Long,
            fileUri: String,
            chapterSort: String,
            volumeFk: Long?,
            isSpecial: Boolean,
        ): ChapterArchive =
            file
                .toChapterArchiveEntity(
                    comicId = comicId,
                    fileUri = fileUri,
                    chapterSort = chapterSort,
                    volumeFk = volumeFk,
                    isSpecial = isSpecial,
                ).copy(checksum = computeChecksum(fileUri))

        /**
         * Também usado por [ChapterSyncService] pra preencher (backfill) o checksum de um
         * capítulo já indexado que ficou com `checksum = null` — sem precisar tratar o
         * capítulo como novo/alterado (sem remapear histórico, sem duplicar linha).
         */
        suspend fun computeChecksum(fileUri: String): String? =
            runCatching {
                DocumentFile.fromSingleUri(context, Uri.parse(fileUri))?.sha256(context)
            }.getOrElse { error ->
                AcerolaLogger.e(TAG, "Failed to hash chapter file: $fileUri", LogSource.REPOSITORY, error)
                null
            }

        companion object {
            private const val TAG = "ChapterIndexer"
        }
    }
