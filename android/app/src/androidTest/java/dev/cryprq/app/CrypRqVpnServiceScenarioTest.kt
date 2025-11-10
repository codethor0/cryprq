package dev.cryprq.app

import android.content.Context
import androidx.test.core.app.ApplicationProvider
import androidx.test.core.app.ServiceScenario
import androidx.test.ext.junit.runners.AndroidJUnit4
import dev.cryprq.tunnel.CrypRqVpnService
import dev.cryprq.tunnel.TunnelStatus
import dev.cryprq.tunnel.TunnelStatusBus
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.take
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class CrypRqVpnServiceScenarioTest {

    private lateinit var context: Context

    @Before
    fun setUp() {
        context = ApplicationProvider.getApplicationContext()
        runBlocking {
            TunnelStatusBus.publish(TunnelStatus.Disconnected)
        }
    }

    @Test
    fun launchingServiceEmitsConnectingAndConnected() = runBlocking {
        val collected = mutableListOf<TunnelStatus>()
        val scenario = ServiceScenario.launch(CrypRqVpnService::class.java)

        withTimeout(2_000) {
            TunnelStatusBus.events
                .filter { it != TunnelStatus.Disconnected }
                .take(2)
                .collect { collected += it }
        }

        scenario.close()
        assertEquals(
            listOf(TunnelStatus.Connecting, TunnelStatus.Connected),
            collected
        )
    }

    @Test
    fun stoppingServiceEmitsDisconnected() = runBlocking {
        val scenario = ServiceScenario.launch(CrypRqVpnService::class.java)
        withTimeout(2_000) {
            TunnelStatusBus.events
                .filter { it == TunnelStatus.Connected }
                .take(1)
                .collect { /* wait for connected */ }
        }

        scenario.close()

        withTimeout(2_000) {
            TunnelStatusBus.events
                .filter { it == TunnelStatus.Disconnected }
                .take(1)
                .collect { /* success */ }
        }
    }
}

