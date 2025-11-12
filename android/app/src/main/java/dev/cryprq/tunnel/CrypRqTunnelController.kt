package dev.cryprq.tunnel

import android.content.Context
import android.content.Intent
import androidx.annotation.StringRes
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dev.cryprq.app.R
import dev.cryprq.logging.CrypRqLogger
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch

class CrypRqTunnelController : ViewModel() {

    private val _status = MutableStateFlow(TunnelStatus.Disconnected)
    val status: StateFlow<TunnelStatus> = _status

    private val _events = MutableStateFlow<List<TunnelEvent>>(emptyList())
    val events: StateFlow<List<TunnelEvent>> = _events

    init {
        viewModelScope.launch {
            TunnelStatusBus.status.collect { status ->
                _status.value = status
                appendEvent(TunnelEvent.StatusChanged(status))
                CrypRqLogger.log("Status -> ${status.name}")
            }
        }
    }

    fun startTunnel(context: Context) {
        appendEvent(TunnelEvent.Message(context.getString(R.string.log_event_starting)))
        CrypRqLogger.log("Start requested from MainActivity")
        val intent = Intent(context, CrypRqVpnService::class.java)
        context.startService(intent)
    }

    fun stopTunnel(context: Context) {
        appendEvent(TunnelEvent.Message(context.getString(R.string.log_event_stopping)))
        CrypRqLogger.log("Stop requested from MainActivity")
        val intent = Intent(context, CrypRqVpnService::class.java)
        context.stopService(intent)
    }

    fun recordInfo(message: String) {
        appendEvent(TunnelEvent.Message(message))
        CrypRqLogger.log(message)
    }

    private fun appendEvent(event: TunnelEvent) {
        val updated = (_events.value + event).takeLast(MAX_EVENTS)
        _events.value = updated
    }

    companion object {
        private const val MAX_EVENTS = 100
    }
}

enum class TunnelStatus(@StringRes val labelRes: Int) {
    Disconnected(R.string.status_disconnected),
    Connecting(R.string.status_connecting),
    Connected(R.string.status_connected)
}

sealed class TunnelEvent(open val timestamp: Long) {
    data class StatusChanged(val status: TunnelStatus, override val timestamp: Long = System.currentTimeMillis()) :
        TunnelEvent(timestamp)

    data class Message(val text: String, override val timestamp: Long = System.currentTimeMillis()) :
        TunnelEvent(timestamp)
}

