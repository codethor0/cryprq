package dev.cryprq.app

import dev.cryprq.tunnel.TunnelStatus
import dev.cryprq.tunnel.TunnelStatusBus
import dev.cryprq.tunnel.CrypRqTunnelController
import dev.cryprq.tunnel.jni.CrypRqNative
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test

class CrypRqNativeTest {
    @get:Rule
    val mainDispatcherRule = MainDispatcherRule()

    @Test
    fun stubReturnsFallbackHandleWhenLibraryMissing() {
        val handle = CrypRqNative.init(CrypRqNative.Config())
        assertEquals(0L, handle)
    }

    @Test
    fun controllerReflectsStatusBusUpdates() = runTest {
        val controller = CrypRqTunnelController()
        TunnelStatusBus.publish(TunnelStatus.Connecting)
        assertEquals(TunnelStatus.Connecting, controller.status.first())

        TunnelStatusBus.publish(TunnelStatus.Connected)
        assertEquals(TunnelStatus.Connected, controller.status.first())
    }
}

