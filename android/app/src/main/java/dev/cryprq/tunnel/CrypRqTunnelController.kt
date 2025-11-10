package dev.cryprq.tunnel

import android.content.Context
import android.content.Intent
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.annotation.StringRes
import dev.cryprq.app.R
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch

class CrypRqTunnelController : ViewModel() {

    private val _status = MutableStateFlow(TunnelStatus.Disconnected)
    val status: StateFlow<TunnelStatus> = _status

    init {
        viewModelScope.launch {
            TunnelStatusBus.status.collect {
                _status.value = it
            }
        }
    }

    fun startTunnel(context: Context) {
        val intent = Intent(context, CrypRqVpnService::class.java)
        context.startService(intent)
    }

    fun stopTunnel(context: Context) {
        val intent = Intent(context, CrypRqVpnService::class.java)
        context.stopService(intent)
    }
}

enum class TunnelStatus(@StringRes val labelRes: Int) {
    Disconnected(R.string.status_disconnected),
    Connecting(R.string.status_connecting),
    Connected(R.string.status_connected)
}

