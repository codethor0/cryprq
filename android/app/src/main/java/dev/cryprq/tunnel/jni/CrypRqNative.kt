package dev.cryprq.tunnel.jni

import android.util.Log

object CrypRqNative {
    private const val TAG = "CrypRqNative"
    private const val ERROR_NOT_AVAILABLE = -1

    var libraryLoaded: Boolean = try {
        System.loadLibrary("cryprq_core")
        true
    } catch (err: UnsatisfiedLinkError) {
        Log.w(TAG, "cryprq_core library not found; JNI calls will fallback to stubs")
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
            Log.w(TAG, "init() called without native library; returning stub handle")
            return 0L
        }
        // TODO: Wire JNI bridge
        Log.w(TAG, "init() native bridge not yet implemented")
        return 0L
    }

    fun connect(handle: Long, params: PeerParams): Int {
        if (!libraryLoaded) return ERROR_NOT_AVAILABLE
        Log.w(TAG, "connect() native bridge not yet implemented")
        return ERROR_NOT_AVAILABLE
    }

    fun readPacket(handle: Long, buffer: ByteArray): Int {
        if (!libraryLoaded) return ERROR_NOT_AVAILABLE
        Log.w(TAG, "readPacket() native bridge not yet implemented")
        return ERROR_NOT_AVAILABLE
    }

    fun writePacket(handle: Long, buffer: ByteArray, len: Int): Int {
        if (!libraryLoaded) return ERROR_NOT_AVAILABLE
        Log.w(TAG, "writePacket() native bridge not yet implemented")
        return ERROR_NOT_AVAILABLE
    }

    fun onNetworkChange(handle: Long): Int {
        if (!libraryLoaded) return ERROR_NOT_AVAILABLE
        Log.w(TAG, "onNetworkChange() native bridge not yet implemented")
        return ERROR_NOT_AVAILABLE
    }

    fun close(handle: Long) {
        if (!libraryLoaded) return
        Log.w(TAG, "close() native bridge not yet implemented")
    }
}

