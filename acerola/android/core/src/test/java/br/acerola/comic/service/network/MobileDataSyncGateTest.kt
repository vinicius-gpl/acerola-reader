package br.acerola.comic.service.network

import android.content.Context
import br.acerola.comic.config.network.isOnCellularConnection
import br.acerola.comic.config.preference.MobileDataSyncPreference
import com.google.common.truth.Truth.assertThat
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkObject
import io.mockk.mockkStatic
import io.mockk.unmockkObject
import io.mockk.unmockkStatic
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class MobileDataSyncGateTest {
    private val context = mockk<Context>(relaxed = true)
    private lateinit var gate: MobileDataSyncGate

    @Before
    fun setup() {
        mockkObject(MobileDataSyncPreference)
        mockkStatic("br.acerola.comic.config.network.NetworkConnectivityKt")
        coEvery { MobileDataSyncPreference.setAlwaysAllow(any(), any()) } returns Unit

        gate = MobileDataSyncGate(context)
    }

    @After
    fun tearDown() {
        unmockkObject(MobileDataSyncPreference)
        unmockkStatic("br.acerola.comic.config.network.NetworkConnectivityKt")
    }

    @Test
    fun `allows right away when not on cellular`() =
        runTest {
            every { MobileDataSyncPreference.alwaysAllowFlow(any()) } returns flowOf(false)
            every { isOnCellularConnection(any()) } returns false

            assertThat(gate.ensureAllowed()).isTrue()
        }

    @Test
    fun `allows right away when the always-allow preference is on`() =
        runTest {
            every { MobileDataSyncPreference.alwaysAllowFlow(any()) } returns flowOf(true)
            every { isOnCellularConnection(any()) } returns true

            assertThat(gate.ensureAllowed()).isTrue()
        }

    @Test
    fun `suspends on cellular without the preference until confirm resolves it`() =
        runTest(UnconfinedTestDispatcher()) {
            every { MobileDataSyncPreference.alwaysAllowFlow(any()) } returns flowOf(false)
            every { isOnCellularConnection(any()) } returns true

            var result: Boolean? = null
            launch { result = gate.ensureAllowed() }

            assertThat(gate.awaitingConfirmation.value).isTrue()
            assertThat(result).isNull()

            gate.confirm(remember = false)

            assertThat(result).isTrue()
            assertThat(gate.awaitingConfirmation.value).isFalse()
        }

    @Test
    fun `cancel resolves ensureAllowed with false`() =
        runTest(UnconfinedTestDispatcher()) {
            every { MobileDataSyncPreference.alwaysAllowFlow(any()) } returns flowOf(false)
            every { isOnCellularConnection(any()) } returns true

            var result: Boolean? = null
            launch { result = gate.ensureAllowed() }

            gate.cancel()

            assertThat(result).isFalse()
        }

    @Test
    fun `confirming with remember persists the always-allow preference`() =
        runTest(UnconfinedTestDispatcher()) {
            every { MobileDataSyncPreference.alwaysAllowFlow(any()) } returns flowOf(false)
            every { isOnCellularConnection(any()) } returns true

            launch { gate.ensureAllowed() }
            gate.confirm(remember = true)

            // A persistência roda no `scope` interno do gate (`Dispatchers.Default` real, não
            // o dispatcher de teste) — timeout em vez de verify direto pra não flakiar numa
            // corrida entre threads de verdade.
            io.mockk.coVerify(timeout = 2000) { MobileDataSyncPreference.setAlwaysAllow(context, true) }
        }
}
