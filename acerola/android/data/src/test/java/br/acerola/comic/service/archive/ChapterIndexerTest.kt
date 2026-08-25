package br.acerola.comic.service.archive

import android.content.Context
import androidx.documentfile.provider.DocumentFile
import br.acerola.comic.util.file.FastFileMetadata
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test

class ChapterIndexerTest {
    private lateinit var indexer: ChapterIndexer
    private val context = mockk<Context>(relaxed = true)

    @Before
    fun setUp() {
        indexer = ChapterIndexer(context)
        mockkStatic(DocumentFile::class)
    }

    @Test
    fun `buildEntity should create ChapterArchive correctly from FastFileMetadata`() =
        runTest {
            // O cálculo de checksum lê via SAF (`DocumentFile.fromSingleUri`), indisponível em um
            // teste unitário simples — anulá-lo aqui isola o comportamento de mapeamento de campos sob teste.
            every { DocumentFile.fromSingleUri(context, any()) } returns null

            val file =
                FastFileMetadata(
                    id = "id1",
                    name = "cap01.cbz",
                    size = 100L,
                    mimeType = "application/x-cbz",
                    lastModified = 1000L,
                )
            val comicId = 10L
            val fileUri = "uri/to/file"
            val chapterSort = "1"
            val volumeFk = 5L
            val isSpecial = false

            val result =
                indexer.buildEntity(
                    file = file,
                    comicId = comicId,
                    fileUri = fileUri,
                    chapterSort = chapterSort,
                    volumeFk = volumeFk,
                    isSpecial = isSpecial,
                )

            assertEquals(comicId, result.folderPathFk)
            assertEquals(fileUri, result.path)
            assertEquals(chapterSort, result.chapterSort)
            assertEquals(volumeFk, result.volumeFk)
            assertEquals(isSpecial, result.isSpecial)
            assertEquals("cap01", result.chapter)
            assertNull(result.checksum)
        }
}
