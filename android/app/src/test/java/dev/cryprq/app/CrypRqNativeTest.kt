package dev.cryprq.app

import dev.cryprq.tunnel.jni.CrypRqNative
import org.junit.Assert.assertEquals
import org.junit.Test

class CrypRqNativeTest {
    @Test
    fun stubReturnsFallbackHandleWhenLibraryMissing() {
        val handle = CrypRqNative.init(CrypRqNative.Config())
        assertEquals(0L, handle)
    }
}

