package br.acerola.comic.service.network

import android.content.Context
import android.net.Uri
import androidx.documentfile.provider.DocumentFile
import br.acerola.comic.adapter.contract.gateway.ComicSingleSyncGateway
import br.acerola.comic.adapter.metadata.comicinfo.ComicInfoEngine
import br.acerola.comic.config.preference.ComicDirectoryPreference
import br.acerola.comic.local.dao.archive.ChapterArchiveDao
import br.acerola.comic.local.dao.archive.ComicDirectoryDao
import br.acerola.comic.local.entity.archive.ComicDirectory
import br.acerola.comic.logging.AcerolaLogger
import br.acerola.comic.logging.LogSource
import br.acerola.comic.pattern.media.MediaFile
import br.acerola.comic.usecase.network.RegisterSyncedChapterUseCase
import br.acerola.comic.util.file.sha256
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import p2p.FfiComicSummaryEntry
import p2p.FfiExtraManifestEntry
import p2p.FfiFileManifestEntry
import p2p.FileSyncProvider
import java.io.IOException
import java.io.InputStream
import java.io.OutputStream
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.atomic.AtomicLong
import javax.inject.Inject
import javax.inject.Singleton

/**
 * Kotlin implementation of [FileSyncProvider] used by the Rust protocol `acerola/sync-files/1`.
 * Reads/writes chapter files (and, desde a extensão pra capa/banner/ComicInfo.xml, itens
 * "extra") via SAF/DocumentFile em chunks — Rust never sees a stream directly, only opaque
 * `Long` handles, so the handle maps here are the "memory" of transfers in flight.
 */
@Singleton
class FileSyncProviderImpl
    @Inject
    constructor(
        @param:ApplicationContext private val context: Context,
        private val comicDirectoryDao: ComicDirectoryDao,
        private val chapterArchiveDao: ChapterArchiveDao,
        private val registerSyncedChapter: RegisterSyncedChapterUseCase,
        @param:ComicInfoEngine private val comicInfoEngine: ComicSingleSyncGateway,
    ) : FileSyncProvider {
        private data class ReadHandle(
            val stream: InputStream,
        )

        /**
         * `target` distingue capítulo de item extra só na hora de FINALIZAR — `writeChapterChunk`
         * (reaproveitado por `beginExtraWrite` também, ver doc da trait em `callbacks.rs` do lado
         * Rust) grava no mesmo `outputStream` independente do que `target` é.
         */
        private sealed class WriteTarget {
            data class Chapter(
                val comicName: String,
                val chapter: String,
            ) : WriteTarget()

            data class Extra(
                val comicName: String,
                val kind: String,
            ) : WriteTarget()
        }

        private data class WriteHandle(
            val target: WriteTarget,
            val fileName: String,
            val expectedChecksum: String,
            val tempFile: DocumentFile,
            val comicFolder: DocumentFile,
            val outputStream: OutputStream,
        )

        private val readHandles = ConcurrentHashMap<Long, ReadHandle>()
        private val writeHandles = ConcurrentHashMap<Long, WriteHandle>()
        private val handleCounter = AtomicLong(1)

        @Volatile
        private var cachedLibraryRoot: DocumentFile? = null

        private suspend fun libraryRoot(): DocumentFile? {
            cachedLibraryRoot?.let { return it }
            val uriString = ComicDirectoryPreference.folderUriFlow(context).first() ?: return null
            val root = DocumentFile.fromTreeUri(context, Uri.parse(uriString)) ?: return null
            cachedLibraryRoot = root
            return root
        }

        // Checksum é calculado no scan da biblioteca (`ChapterIndexer.buildEntity`), não aqui —
        // ler o arquivo inteiro pra hashear na hora do handshake P2P é o pior momento possível
        // pra isso (era o que estourava o timeout de 30s do outro lado numa biblioteca grande
        // ou recém-escaneada). Um capítulo sem checksum cacheado (biblioteca escaneada antes
        // dessa mudança, ainda sem rescan) fica de fora do manifesto — melhor omitir do que
        // bloquear a sessão inteira; o próximo rescan preenche.
        override fun getFileManifest(): List<FfiFileManifestEntry> =
            runBlocking {
                comicDirectoryDao.getAllDirectories().first().flatMap { comic ->
                    chapterArchiveDao.getChaptersListByDirectoryId(comic.id).map { chapter ->
                        async(Dispatchers.IO) {
                            val checksum = chapter.checksum ?: return@async null
                            val file = DocumentFile.fromSingleUri(context, Uri.parse(chapter.path))
                            if (file == null || !file.exists()) return@async null

                            FfiFileManifestEntry(
                                comicName = comic.name,
                                chapter = chapter.chapter,
                                fileName = file.name ?: chapter.chapter,
                                checksum = checksum,
                                sizeBytes = file.length().toULong(),
                            )
                        }
                    }.awaitAll().filterNotNull()
                }
            }

        // Só Room/SQLite — nenhum `DocumentFile`/SAF aqui, de propósito. `getFileManifest()`
        // acima paga um `DocumentFile.exists()` por capítulo (uma transação binder cada) porque
        // precisa apontar pra um arquivo de verdade pra transferência; `browse-library` só
        // precisa de nome + contagem, então usa a consulta agregada direto no DB e nunca toca
        // SAF — é o mesmo custo que estourava o timeout do protocolo antes dessa separação.
        override fun getLibrarySummary(): List<FfiComicSummaryEntry> =
            runBlocking {
                chapterArchiveDao.getLibrarySummary().map { row ->
                    FfiComicSummaryEntry(
                        comicName = row.comicName,
                        chapterCount = row.chapterCount.toUInt(),
                        coverVersion = row.coverVersion,
                    )
                }
            }

        override fun openChapterForRead(
            comicName: String,
            chapter: String,
        ): Long =
            runBlocking {
                val comic = comicDirectoryDao.getDirectoryByName(comicName) ?: return@runBlocking -1L
                val chapterArchive =
                    chapterArchiveDao.getChapterByDirectoryAndChapter(comic.id, chapter) ?: return@runBlocking -1L
                val stream =
                    context.contentResolver.openInputStream(Uri.parse(chapterArchive.path)) ?: return@runBlocking -1L

                val handle = handleCounter.getAndIncrement()
                readHandles[handle] = ReadHandle(stream)
                handle
            }

        override fun readChapterChunk(
            handle: Long,
            chunkSize: UInt,
        ): ByteArray {
            val readHandle = readHandles[handle] ?: return ByteArray(0)
            val buffer = ByteArray(chunkSize.toInt())

            // `InputStream.read()` só sinaliza fim de arquivo com -1 — 0 é "nenhum byte
            // disponível agora, mas não acabou" (possível num stream cross-process via SAF).
            // Tratar 0 como fim (como estava antes) fazia o remetente parar de mandar bytes
            // NO MEIO do capítulo — o lado que recebe fica esperando o resto, e a stream
            // desalinha: os próximos bytes que chegam são o header JSON do PRÓXIMO capítulo,
            // interpretado por engano como mais conteúdo binário do capítulo atual.
            var read = readHandle.stream.read(buffer)
            while (read == 0) {
                read = readHandle.stream.read(buffer)
            }
            if (read == -1) return ByteArray(0)
            return if (read == buffer.size) buffer else buffer.copyOf(read)
        }

        override fun closeReadHandle(handle: Long) {
            runCatching { readHandles.remove(handle)?.stream?.close() }
        }

        override fun beginChapterWrite(
            comicName: String,
            chapter: String,
            fileName: String,
            expectedChecksum: String,
            sizeBytes: ULong,
        ): Long =
            runBlocking {
                val root = libraryRoot() ?: return@runBlocking -1L

                // Se o quadrinho já existe localmente, o capítulo tem que cair na MESMA pasta
                // que os capítulos já existentes (`comic.path`). Quadrinho novo cai direto em
                // `<root>/<comicName>/` — mesmo lugar que o DirectoryScanner criaria se o
                // usuário tivesse adicionado manualmente. Sem pasta `synced/` intermediária:
                // ela não tinha nenhuma exclusão no DirectoryScanner (que desce recursivo em
                // qualquer pasta sem manga direto nela), então um rescan completo podia
                // redescobrir `synced/<comicName>/` como um segundo ComicDirectory fantasma.
                val existingComic = comicDirectoryDao.getDirectoryByName(comicName)
                val comicFolder =
                    if (existingComic != null) {
                        DocumentFile.fromTreeUri(context, Uri.parse(existingComic.path))
                    } else {
                        root.findFile(comicName) ?: root.createDirectory(comicName)
                    }
                if (comicFolder == null) return@runBlocking -1L

                // Final name comes from the peer (preserves the original .cbz/.cbr extension);
                // if it comes empty (the "chapter unavailable" placeholder), fall back to the
                // chapter label.
                val finalName = fileName.ifBlank { chapter }
                val tempName = "$finalName.part"
                comicFolder.findFile(tempName)?.delete()
                val tempFile =
                    comicFolder.createFile("application/octet-stream", tempName) ?: return@runBlocking -1L
                val outputStream =
                    context.contentResolver.openOutputStream(tempFile.uri, "wt") ?: run {
                        tempFile.delete()
                        return@runBlocking -1L
                    }

                val handle = handleCounter.getAndIncrement()
                writeHandles[handle] =
                    WriteHandle(
                        WriteTarget.Chapter(comicName, chapter),
                        finalName,
                        expectedChecksum,
                        tempFile,
                        comicFolder,
                        outputStream,
                    )
                handle
            }

        override fun writeChapterChunk(
            handle: Long,
            bytes: ByteArray,
        ): Boolean {
            val writeHandle = writeHandles[handle] ?: return false
            return try {
                writeHandle.outputStream.write(bytes)
                true
            } catch (error: IOException) {
                AcerolaLogger.e("FileSyncProvider", "Failed to write chunk for handle $handle", LogSource.NETWORK, error)
                false
            }
        }

        /**
         * Fecha o stream, verifica o checksum do arquivo temporário e remove o handle do mapa —
         * compartilhado entre `finalizeChapterWrite`/`finalizeExtraWrite`, a única diferença
         * entre os dois é o que acontece DEPOIS do checksum bater (ver `target` de cada um).
         * `null` cobre tanto "handle desconhecido" quanto "checksum não bateu" (arquivo
         * temporário já deletado nesse segundo caso).
         */
        private fun finalizeAndVerifyChecksum(handle: Long): WriteHandle? {
            val writeHandle = writeHandles.remove(handle) ?: return null
            runCatching { writeHandle.outputStream.close() }

            val actualChecksum = writeHandle.tempFile.sha256(context)
            if (actualChecksum != writeHandle.expectedChecksum) {
                writeHandle.tempFile.delete()
                return null
            }

            return writeHandle
        }

        override fun finalizeChapterWrite(handle: Long): Boolean =
            runBlocking {
                val writeHandle = finalizeAndVerifyChecksum(handle) ?: return@runBlocking false
                val target = writeHandle.target as? WriteTarget.Chapter ?: return@runBlocking false

                writeHandle.comicFolder.findFile(writeHandle.fileName)?.delete()
                if (!writeHandle.tempFile.renameTo(writeHandle.fileName)) {
                    writeHandle.tempFile.delete()
                    return@runBlocking false
                }

                val finalFile = writeHandle.comicFolder.findFile(writeHandle.fileName) ?: writeHandle.tempFile
                registerSyncedChapter(
                    comicName = target.comicName,
                    chapter = target.chapter,
                    checksum = writeHandle.expectedChecksum,
                    fileUri = finalFile.uri.toString(),
                    comicFolderUri = writeHandle.comicFolder.uri.toString(),
                )
                true
            }

        override fun abortChapterWrite(handle: Long) {
            val writeHandle = writeHandles.remove(handle) ?: return
            runCatching { writeHandle.outputStream.close() }
            writeHandle.tempFile.delete()
        }

        // --- Itens extra (capa/banner/ComicInfo.xml) ---
        //
        // `openExtraForRead`/`beginExtraWrite` inserem nos MESMOS mapas (`readHandles`/
        // `writeHandles`) usados por capítulo — `readChapterChunk`/`writeChapterChunk`/
        // `closeReadHandle` são reaproveitados sem mudança (ver doc da trait em `callbacks.rs`
        // do lado Rust), então os handles de extra têm que viver no mesmo espaço pra essas três
        // funções conseguirem achá-los.

        override fun getExtrasManifest(): List<FfiExtraManifestEntry> =
            runBlocking {
                comicDirectoryDao.getAllDirectories().first().map { comic ->
                    async(Dispatchers.IO) { buildExtraEntries(comic) }
                }.awaitAll().flatten()
            }

        private fun buildExtraEntries(comic: ComicDirectory): List<FfiExtraManifestEntry> {
            val entries = mutableListOf<FfiExtraManifestEntry>()

            comic.cover?.let { path -> artworkExtraEntry(comic.name, EXTRA_KIND_COVER, path)?.let(entries::add) }
            comic.banner?.let { path -> artworkExtraEntry(comic.name, EXTRA_KIND_BANNER, path)?.let(entries::add) }
            findComicInfoFile(comic)?.let { file ->
                runCatching {
                    entries.add(
                        FfiExtraManifestEntry(
                            comicName = comic.name,
                            kind = EXTRA_KIND_COMIC_INFO,
                            fileName = file.name ?: COMIC_INFO_FILE_NAME,
                            checksum = file.sha256(context),
                            sizeBytes = file.length().toULong(),
                        ),
                    )
                }
            }

            return entries
        }

        private fun artworkExtraEntry(
            comicName: String,
            kind: String,
            path: String,
        ): FfiExtraManifestEntry? {
            val file = DocumentFile.fromSingleUri(context, Uri.parse(path))
            if (file == null || !file.exists()) return null

            return runCatching {
                FfiExtraManifestEntry(
                    comicName = comicName,
                    kind = kind,
                    fileName = file.name ?: MediaFile.COVER.defaultFileName,
                    checksum = file.sha256(context),
                    sizeBytes = file.length().toULong(),
                )
            }.getOrNull()
        }

        /**
         * Acha `ComicInfo.xml` (case-insensitive) direto na raiz da pasta do quadrinho — mesma
         * convenção do Desktop (`core/services/metadata/mod.rs::find_comic_info_path`).
         */
        private fun findComicInfoFile(comic: ComicDirectory): DocumentFile? {
            val comicFolder = DocumentFile.fromTreeUri(context, Uri.parse(comic.path)) ?: return null
            return comicFolder.listFiles().firstOrNull {
                it.isFile && it.name?.equals(COMIC_INFO_FILE_NAME, ignoreCase = true) == true
            }
        }

        override fun openExtraForRead(
            comicName: String,
            kind: String,
        ): Long =
            runBlocking {
                val comic = comicDirectoryDao.getDirectoryByName(comicName) ?: return@runBlocking -1L

                val sourceFile =
                    when (kind) {
                        EXTRA_KIND_COVER -> comic.cover?.let { DocumentFile.fromSingleUri(context, Uri.parse(it)) }
                        EXTRA_KIND_BANNER -> comic.banner?.let { DocumentFile.fromSingleUri(context, Uri.parse(it)) }
                        EXTRA_KIND_COMIC_INFO -> findComicInfoFile(comic)
                        else -> null
                    } ?: return@runBlocking -1L

                val stream = context.contentResolver.openInputStream(sourceFile.uri) ?: return@runBlocking -1L

                val handle = handleCounter.getAndIncrement()
                readHandles[handle] = ReadHandle(stream)
                handle
            }

        override fun beginExtraWrite(
            comicName: String,
            kind: String,
            fileName: String,
            expectedChecksum: String,
            sizeBytes: ULong,
        ): Long =
            runBlocking {
                val root = libraryRoot() ?: return@runBlocking -1L

                val existingComic = comicDirectoryDao.getDirectoryByName(comicName)
                val comicFolder =
                    if (existingComic != null) {
                        DocumentFile.fromTreeUri(context, Uri.parse(existingComic.path))
                    } else {
                        root.findFile(comicName) ?: root.createDirectory(comicName)
                    }
                if (comicFolder == null) return@runBlocking -1L

                // Nome final canônico por `kind` — capa/banner sempre `.jpg` (mesma convenção já
                // usada por `CoverSaver`/`BannerSaver` no fluxo manual de metadata, independente
                // da extensão que o peer anunciou), `ComicInfo.xml` sempre esse nome exato.
                val finalName =
                    when (kind) {
                        EXTRA_KIND_COVER -> MediaFile.COVER.defaultFileName
                        EXTRA_KIND_BANNER -> MediaFile.BANNER.defaultFileName
                        EXTRA_KIND_COMIC_INFO -> COMIC_INFO_FILE_NAME
                        else -> return@runBlocking -1L
                    }

                val tempName = "$finalName.part"
                comicFolder.findFile(tempName)?.delete()
                val tempFile =
                    comicFolder.createFile("application/octet-stream", tempName) ?: return@runBlocking -1L
                val outputStream =
                    context.contentResolver.openOutputStream(tempFile.uri, "wt") ?: run {
                        tempFile.delete()
                        return@runBlocking -1L
                    }

                val handle = handleCounter.getAndIncrement()
                writeHandles[handle] =
                    WriteHandle(WriteTarget.Extra(comicName, kind), finalName, expectedChecksum, tempFile, comicFolder, outputStream)
                handle
            }

        override fun finalizeExtraWrite(handle: Long): Boolean =
            runBlocking {
                val writeHandle = finalizeAndVerifyChecksum(handle) ?: return@runBlocking false
                val target = writeHandle.target as? WriteTarget.Extra ?: return@runBlocking false

                when (target.kind) {
                    EXTRA_KIND_COVER -> finalizeArtworkExtra(writeHandle, target, MediaFile.COVER)
                    EXTRA_KIND_BANNER -> finalizeArtworkExtra(writeHandle, target, MediaFile.BANNER)
                    EXTRA_KIND_COMIC_INFO -> finalizeComicInfoExtra(writeHandle, target)
                    else -> {
                        writeHandle.tempFile.delete()
                        false
                    }
                }
            }

        private suspend fun finalizeArtworkExtra(
            writeHandle: WriteHandle,
            target: WriteTarget.Extra,
            media: MediaFile,
        ): Boolean {
            val directory =
                comicDirectoryDao.getDirectoryByName(target.comicName) ?: run {
                    writeHandle.tempFile.delete()
                    return false
                }

            // Limpa qualquer cover/banner pré-existente antes de gravar o novo — mesma limpeza
            // de órfão que `CoverSaver`/`BannerSaver` já fazem pro fluxo manual de metadata.
            writeHandle.comicFolder.listFiles().forEach { file ->
                val name = file.name ?: return@forEach
                val matches = if (media == MediaFile.COVER) MediaFile.isCover(name) else MediaFile.isBanner(name)
                if (matches) file.delete()
            }

            if (!writeHandle.tempFile.renameTo(writeHandle.fileName)) {
                writeHandle.tempFile.delete()
                return false
            }

            val finalFile = writeHandle.comicFolder.findFile(writeHandle.fileName) ?: writeHandle.tempFile
            val savedUri = finalFile.uri.toString()
            val updated =
                if (media == MediaFile.COVER) {
                    directory.copy(cover = savedUri, lastModified = System.currentTimeMillis())
                } else {
                    directory.copy(banner = savedUri, lastModified = System.currentTimeMillis())
                }
            comicDirectoryDao.update(updated)
            return true
        }

        private suspend fun finalizeComicInfoExtra(
            writeHandle: WriteHandle,
            target: WriteTarget.Extra,
        ): Boolean {
            val directory =
                comicDirectoryDao.getDirectoryByName(target.comicName) ?: run {
                    writeHandle.tempFile.delete()
                    return false
                }

            writeHandle.comicFolder.listFiles()
                .filter { it.isFile && it.name?.equals(writeHandle.fileName, ignoreCase = true) == true }
                .forEach { it.delete() }

            if (!writeHandle.tempFile.renameTo(writeHandle.fileName)) {
                writeHandle.tempFile.delete()
                return false
            }

            // Melhor esforço: o arquivo já foi persistido com sucesso mesmo se o
            // reprocessamento de metadata falhar — mesma postura do Desktop
            // (`infra/sync/protocol/transfer.rs::receive_extras`).
            runCatching { comicInfoEngine.refreshManga(comicId = directory.id, baseUri = null) }
                .onFailure { error ->
                    AcerolaLogger.e(
                        "FileSyncProvider",
                        "ComicInfo.xml recebido, mas reprocessamento de metadata falhou",
                        LogSource.NETWORK,
                        error,
                    )
                }

            return true
        }

        override fun abortExtraWrite(handle: Long) = abortChapterWrite(handle)

        companion object {
            private const val EXTRA_KIND_COVER = "cover"
            private const val EXTRA_KIND_BANNER = "banner"
            private const val EXTRA_KIND_COMIC_INFO = "comic_info"
            private const val COMIC_INFO_FILE_NAME = "ComicInfo.xml"
        }
    }
