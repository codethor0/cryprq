package dev.cryprq.tunnel

import android.app.NotificationManager
import android.content.Intent
import android.net.VpnService
import android.os.Build
import android.os.ParcelFileDescriptor
import android.util.Log
import dev.cryprq.app.R
import dev.cryprq.logging.CrypRqLogger
import dev.cryprq.tunnel.jni.CrypRqNative
import dev.cryprq.tunnel.notifications.ForegroundNotification
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch

class CrypRqVpnService : VpnService() {

    private val job = SupervisorJob()
    private val scope = CoroutineScope(job + Dispatchers.IO)
    private var handle: Long = 0L
    private var tunFd: ParcelFileDescriptor? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        CrypRqLogger.init(applicationContext)
        if (intent?.action == ACTION_STOP) {
            stopTunnel("User requested stop")
            return START_NOT_STICKY
        }
        CrypRqLogger.log("CrypRqVpnService started")
        scope.launch { TunnelStatusBus.publish(TunnelStatus.Connecting) }
        startForeground(NOTIFICATION_ID, ForegroundNotification.build(this))

        establishTunnel()
        connectNative()

        return START_STICKY
    }

    override fun onRevoke() {
        super.onRevoke()
        stopTunnel("VPN permissions revoked")
    }

    override fun onDestroy() {
        super.onDestroy()
        stopTunnel("Service destroyed")
        job.cancel()
    }

    private fun establishTunnel() {
        val configureIntent = android.app.PendingIntent.getActivity(
            this,
            0,
            Intent(this, dev.cryprq.app.MainActivity::class.java),
            android.app.PendingIntent.FLAG_IMMUTABLE or android.app.PendingIntent.FLAG_UPDATE_CURRENT
        )

        val builder = Builder()
            .setSession(getString(R.string.app_name))
            .setConfigureIntent(configureIntent)
            .setMtu(DEFAULT_MTU)

        builder.addAddress("10.0.0.2", 32)
        builder.addRoute("0.0.0.0", 0)
        requestNotificationPermissionIfNeeded()

        tunFd = builder.establish()
        if (tunFd == null) {
            Log.e(TAG, "Failed to establish TUN interface")
            CrypRqLogger.log("Failed to establish TUN interface")
            stopSelf()
        }
    }

    private fun connectNative() {
        if (!CrypRqNative.libraryLoaded) {
            Log.w(TAG, "Native library not available; running in dry-run mode")
            CrypRqLogger.log("Native library unavailable; running in dry-run mode")
            scope.launch { TunnelStatusBus.publish(TunnelStatus.Connected) }
            return
        }

        handle = CrypRqNative.init(
            CrypRqNative.Config(
                logLevel = "info",
                allowPeers = emptyList()
            )
        )
        val code = CrypRqNative.connect(
            handle,
            CrypRqNative.PeerParams(
                mode = CrypRqNative.MODE_LISTENER,
                multiaddr = DEFAULT_MULTIADDR
            )
        )
        if (code == 0) {
            scope.launch { TunnelStatusBus.publish(TunnelStatus.Connected) }
            CrypRqLogger.log("cryprq_connect succeeded")
        } else {
            Log.e(TAG, "cryprq_connect returned $code")
            CrypRqLogger.log("cryprq_connect failed with code $code")
            stopTunnel("connect failed ($code)")
        }
    }

    private fun stopTunnel(reason: String) {
        Log.i(TAG, "Stopping tunnel: $reason")
        CrypRqLogger.log("Stopping tunnel: $reason")
        tunFd?.close()
        tunFd = null

        if (handle != 0L) {
            CrypRqNative.close(handle)
            handle = 0L
        }
        scope.launch { TunnelStatusBus.publish(TunnelStatus.Disconnected) }
        stopForeground(STOP_FOREGROUND_REMOVE)
        stopSelf()
    }

    private fun requestNotificationPermissionIfNeeded() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            val manager = getSystemService(NotificationManager::class.java)
            if (!manager.areNotificationsEnabled()) {
                Log.w(TAG, "Notifications disabled; foreground service may be hidden")
            }
        }
    }

    companion object {
        private const val TAG = "CrypRqVpnService"
        private const val NOTIFICATION_ID = 42
        private const val DEFAULT_MTU = 1400
        private const val DEFAULT_MULTIADDR = "/ip4/0.0.0.0/udp/9999/quic-v1"
        const val ACTION_STOP = "dev.cryprq.tunnel.action.STOP"
    }
}

