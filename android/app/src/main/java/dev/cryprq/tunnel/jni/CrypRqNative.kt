package dev.cryprq.tunnel.jni

import android.util.Log

object CrypRqNative {
    private const val TAG = "CrypRqNative"
    private const val ERROR_NOT_AVAILABLE = -1

    var libraryLoaded: Boolean = try {
        System.loadLibrary("cryprq_android")
        true
    } catch (err: UnsatisfiedLinkError) {
        Log.w(TAG, "cryprq_android library not found; JNI calls will fallback to stubs")
        false
    }

    data class Config(
        val logLevel: String? = null,
        val allowPeers: List<String> = emptyList()
    )

    data class PeerParams(
        val mode: Int,
        val multiaddr: String
    )

    const val MODE_LISTENER = 0
    const val MODE_DIAL = 1

    fun init(config: Config): Long {
        if (!libraryLoaded) {
            Log.w(TAG, "init() called without native bridge; returning stub handle")
            return 0L
        }
        return nativeInit(config.logLevel, config.allowPeers.toTypedArray())
    }

    fun connect(handle: Long, params: PeerParams): Int {
        if (!libraryLoaded) return ERROR_NOT_AVAILABLE
        return nativeConnect(handle, params.mode, params.multiaddr)
    }

    fun readPacket(handle: Long, buffer: ByteArray): Int {
        if (!libraryLoaded) return ERROR_NOT_AVAILABLE
        return nativeReadPacket(handle, buffer)
    }

    fun writePacket(handle: Long, buffer: ByteArray, len: Int): Int {
        if (!libraryLoaded) return ERROR_NOT_AVAILABLE
        return nativeWritePacket(handle, buffer, len)
    }

    fun onNetworkChange(handle: Long): Int {
        if (!libraryLoaded) return ERROR_NOT_AVAILABLE
        return nativeOnNetworkChange(handle)
    }

    fun close(handle: Long) {
        if (!libraryLoaded) return
        nativeClose(handle)
    }

    @JvmStatic
    private external fun nativeInit(logLevel: String?, allowPeers: Array<String>): Long

    @JvmStatic
    private external fun nativeConnect(handle: Long, mode: Int, multiaddr: String): Int

    @JvmStatic
    private external fun nativeReadPacket(handle: Long, buffer: ByteArray): Int

    @JvmStatic
    private external fun nativeWritePacket(handle: Long, buffer: ByteArray, len: Int): Int

    @JvmStatic
    private external fun nativeOnNetworkChange(handle: Long): Int

    @JvmStatic
    private external fun nativeClose(handle: Long)
}

