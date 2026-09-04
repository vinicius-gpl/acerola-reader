package br.acerola.comic

import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import p2p.FfiChapterReadEntry
import p2p.FfiComicSummaryEntry
import p2p.FfiFileManifestEntry
import p2p.FfiReadingProgressEntry
import p2p.FileSyncProvider
import p2p.HistorySyncProvider
import p2p.P2pCallback
import p2p.P2pNode
import p2p.SecureBlobStore
import java.io.ByteArrayOutputStream
import java.security.MessageDigest
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.CopyOnWriteArrayList
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicLong

private val FILE_SYNC_ALPN = "acerola/sync-files/1".toByteArray()

private fun sha256(bytes: ByteArray): String = MessageDigest.getInstance("SHA-256").digest(bytes).joinToString(separator = "") { "%02x".format(it) }

/** Determinístico e variando com `seed` — capítulos diferentes não têm o mesmo conteúdo,
 *  pra não mascarar um bug de troca/mistura entre capítulos. */
private fun makePayload(
    size: Int,
    seed: Int,
): ByteArray = ByteArray(size) { i -> (((i.toLong() * 31 + seed) % 251)).toByte() }

/**
 * Testes de integridade do protocolo `acerola/sync-files/1` que atravessam a FFI/JNA de
 * verdade — dois [P2pNode] reais, rodando o `libacerola.so` compilado pro dispositivo (não
 * um mock Rust puro como em `native/rust/.../exchange.rs::tests`), conectados um ao outro.
 *
 * Existe pra fechar uma lacuna deixada pelos testes puramente Rust (`exchange.rs`): aqueles
 * provam que a lógica de chunking/framing/checksum está correta DENTRO do Rust, mas não
 * conseguem pegar um bug que só existe na travessia real Kotlin<->Rust (marshaling de
 * String/ByteArray via UniFFI/JNA) — que é exatamente o trecho que ainda não tinha cobertura
 * nenhuma na investigação do bug de "checksum mismatch" em produção.
 *
 * [InMemoryFileSyncProvider] guarda bytes em memória em vez de tocar SAF de propósito —
 * isola "a FFI + protocolo Rust<->Kotlin corrompe o checksum" de "o SAF/ContentResolver tem
 * um bug", que é uma camada totalmente diferente ([FileSyncProviderImpl] real, não coberta
 * aqui).
 */
@RunWith(AndroidJUnit4::class)
class FileSyncFfiIntegrationTest {
    private class NoOpHistoryProvider : HistorySyncProvider {
        override fun getReadingProgress(): List<FfiReadingProgressEntry> = emptyList()

        override fun getChaptersRead(): List<FfiChapterReadEntry> = emptyList()

        override fun applyReadingProgress(entry: FfiReadingProgressEntry): Boolean = false

        override fun applyChapterRead(entry: FfiChapterReadEntry): Boolean = false
    }

    private class InMemorySecureBlobStore : SecureBlobStore {
        private val store = ConcurrentHashMap<String, ByteArray>()

        override fun saveBlob(
            key: String,
            value: ByteArray,
        ) {
            store[key] = value
        }

        override fun loadBlob(key: String): ByteArray? = store[key]
    }

    private data class FinalizedChapter(
        val comicName: String,
        val chapter: String,
        val expectedChecksum: String,
        val actualChecksum: String,
        val bytes: ByteArray,
    )

    private class InMemoryFileSyncProvider(
        private val readable: Map<Pair<String, String>, Pair<String, ByteArray>>,
    ) : FileSyncProvider {
        private data class ReadState(
            val bytes: ByteArray,
            var position: Int,
        )

        private data class WriteState(
            val comicName: String,
            val chapter: String,
            val expectedChecksum: String,
            val buffer: ByteArrayOutputStream = ByteArrayOutputStream(),
        )

        private val readHandles = ConcurrentHashMap<Long, ReadState>()
        private val writeHandles = ConcurrentHashMap<Long, WriteState>()
        private val nextHandle = AtomicLong(1)
        val finalized = CopyOnWriteArrayList<FinalizedChapter>()

        override fun getFileManifest(): List<FfiFileManifestEntry> =
            readable.map { (key, value) ->
                val (comicName, chapter) = key
                val (fileName, bytes) = value
                FfiFileManifestEntry(comicName, chapter, fileName, sha256(bytes), bytes.size.toULong())
            }

        override fun getLibrarySummary(): List<FfiComicSummaryEntry> =
            readable.keys
                .groupingBy { (comicName, _) -> comicName }
                .eachCount()
                .map { (comicName, count) -> FfiComicSummaryEntry(comicName, count.toUInt()) }

        override fun openChapterForRead(
            comicName: String,
            chapter: String,
        ): Long {
            val entry = readable[comicName to chapter] ?: return -1L
            val handle = nextHandle.getAndIncrement()
            readHandles[handle] = ReadState(entry.second, 0)
            return handle
        }

        override fun readChapterChunk(
            handle: Long,
            chunkSize: UInt,
        ): ByteArray {
            val state = readHandles[handle] ?: return ByteArray(0)
            val end = minOf(state.position + chunkSize.toInt(), state.bytes.size)
            if (state.position >= end) return ByteArray(0)
            val chunk = state.bytes.copyOfRange(state.position, end)
            state.position = end
            return chunk
        }

        override fun closeReadHandle(handle: Long) {
            readHandles.remove(handle)
        }

        override fun beginChapterWrite(
            comicName: String,
            chapter: String,
            fileName: String,
            expectedChecksum: String,
            sizeBytes: ULong,
        ): Long {
            val handle = nextHandle.getAndIncrement()
            writeHandles[handle] = WriteState(comicName, chapter, expectedChecksum)
            return handle
        }

        override fun writeChapterChunk(
            handle: Long,
            bytes: ByteArray,
        ): Boolean {
            val state = writeHandles[handle] ?: return false
            state.buffer.write(bytes)
            return true
        }

        override fun finalizeChapterWrite(handle: Long): Boolean {
            val state = writeHandles.remove(handle) ?: return false
            val bytes = state.buffer.toByteArray()
            val actualChecksum = sha256(bytes)
            finalized.add(FinalizedChapter(state.comicName, state.chapter, state.expectedChecksum, actualChecksum, bytes))
            return actualChecksum == state.expectedChecksum
        }

        override fun abortChapterWrite(handle: Long) {
            writeHandles.remove(handle)
        }
    }

    /**
     * Dois nós reais, cada um oferecendo um capítulo que o outro não tem — dispara
     * transferência bidirecional numa única sessão `acerola/sync-files/1`, atravessando a
     * FFI/JNA de verdade nos dois sentidos (Kotlin->Rust ao montar o manifesto local,
     * Rust->Kotlin ao declarar o checksum esperado pro lado que recebe). Compara tanto o
     * checksum (SHA-256 real, mesmo algoritmo de produção) quanto os bytes byte-a-byte.
     */
    @Test
    fun realFfiRoundTrip_transfersRealBytesWithCorrectChecksum() {
        val payloadA = makePayload(1_500_037, 7)
        val payloadB = makePayload(900_321, 42)

        val providerA = InMemoryFileSyncProvider(mapOf(("Comic A" to "Ch. 1") to ("ch1.cbz" to payloadA)))
        val providerB = InMemoryFileSyncProvider(mapOf(("Comic B" to "Ch. 1") to ("ch1.cbz" to payloadB)))

        val eventsA = CopyOnWriteArrayList<Pair<String, String>>()
        val eventsB = CopyOnWriteArrayList<Pair<String, String>>()
        val latch = CountDownLatch(2)

        val callbackA =
            object : P2pCallback {
                override fun onEvent(
                    event: String,
                    data: String,
                ) {
                    eventsA.add(event to data)
                    if (event == "sync:files:complete") latch.countDown()
                }
            }
        val callbackB =
            object : P2pCallback {
                override fun onEvent(
                    event: String,
                    data: String,
                ) {
                    eventsB.add(event to data)
                    if (event == "sync:files:complete") latch.countDown()
                }
            }

        val nodeA =
            P2pNode(
                callbackA,
                null,
                null,
                "ffi-test-device-a",
                "1.0",
                InMemorySecureBlobStore(),
                NoOpHistoryProvider(),
                providerA,
            )
        val nodeB =
            P2pNode(
                callbackB,
                null,
                null,
                "ffi-test-device-b",
                "1.0",
                InMemorySecureBlobStore(),
                NoOpHistoryProvider(),
                providerB,
            )

        try {
            val addrB = nodeB.getLocalAddr()
            nodeA.connect(addrB, FILE_SYNC_ALPN)

            val completed = latch.await(60, TimeUnit.SECONDS)
            assertTrue(
                "sync did not complete on both sides within 60s. Events A: $eventsA | Events B: $eventsB",
                completed,
            )

            assertEquals("A should have received exactly 1 chapter from B", 1, providerA.finalized.size)
            val receivedByA = providerA.finalized[0]
            assertEquals("Comic B", receivedByA.comicName)
            assertEquals(
                "checksum declared by B does not match recalculated by A after receiving",
                receivedByA.expectedChecksum,
                receivedByA.actualChecksum,
            )
            assertArrayEquals("bytes received by A do not match byte-by-byte with what B sent", payloadB, receivedByA.bytes)

            assertEquals("B should have received exactly 1 chapter from A", 1, providerB.finalized.size)
            val receivedByB = providerB.finalized[0]
            assertEquals("Comic A", receivedByB.comicName)
            assertEquals(
                "checksum declared by A does not match recalculated by B after receiving",
                receivedByB.expectedChecksum,
                receivedByB.actualChecksum,
            )
            assertArrayEquals("bytes received by B do not match byte-by-byte with what A sent", payloadA, receivedByB.bytes)
        } finally {
            nodeA.shutdown()
            nodeB.shutdown()
        }
    }

    /**
     * Mesma ideia do teste de stress em `exchange.rs`, mas atravessando a FFI real: muitos
     * capítulos de tamanhos variados numa única sessão sequencial. Alvo específico: um
     * handle de leitura/escrita sendo reaproveitado incorretamente do lado Kotlin (os mapas
     * `ConcurrentHashMap<Long, ...>` em [FileSyncProviderImpl] real usam o mesmo padrão de
     * handle opaco que este mock) — se houvesse vazamento de estado entre capítulos
     * consecutivos, apareceria aqui como bytes de um capítulo indo parar em outro.
     */
    @Test
    fun realFfiRoundTrip_stressManySequentialChaptersPreserveChecksumIntegrity() {
        val chapterCount = 40
        val entries =
            (0 until chapterCount).associate { i ->
                val size = 10_000 + (i * 37_123) % 500_000
                ("Stress Comic" to "Ch. $i") to ("ch$i.cbz" to makePayload(size, i))
            }

        val sender = InMemoryFileSyncProvider(entries)
        val receiver = InMemoryFileSyncProvider(emptyMap())

        val latch = CountDownLatch(2)
        val events = CopyOnWriteArrayList<Pair<String, String>>()
        val callback =
            object : P2pCallback {
                override fun onEvent(
                    event: String,
                    data: String,
                ) {
                    events.add(event to data)
                    if (event == "sync:files:complete") latch.countDown()
                }
            }

        val nodeSender =
            P2pNode(callback, null, null, "ffi-stress-sender", "1.0", InMemorySecureBlobStore(), NoOpHistoryProvider(), sender)
        val nodeReceiver =
            P2pNode(callback, null, null, "ffi-stress-receiver", "1.0", InMemorySecureBlobStore(), NoOpHistoryProvider(), receiver)

        try {
            nodeSender.connect(nodeReceiver.getLocalAddr(), FILE_SYNC_ALPN)

            val completed = latch.await(120, TimeUnit.SECONDS)
            assertTrue("sync did not complete within 120s. Events: $events", completed)

            assertEquals("not all chapters arrived", chapterCount, receiver.finalized.size)

            val byChapter = receiver.finalized.associateBy { it.chapter }
            entries.forEach { (key, value) ->
                val (comicName, chapter) = key
                val (_, expectedBytes) = value
                val got = byChapter[chapter] ?: error("chapter $chapter never arrived at the receiving side")
                assertEquals("chapter $chapter: comic_name diverged", comicName, got.comicName)
                assertEquals(
                    "chapter $chapter: declared checksum does not match recalculated",
                    got.expectedChecksum,
                    got.actualChecksum,
                )
                assertArrayEquals(
                    "chapter $chapter: received bytes diverge from original (possible state leak between handles)",
                    expectedBytes,
                    got.bytes,
                )
            }
        } finally {
            nodeSender.shutdown()
            nodeReceiver.shutdown()
        }
    }
}
