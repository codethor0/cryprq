package dev.cryprq.tunnel

import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow

object TunnelStatusBus {
    private val _status = MutableStateFlow(TunnelStatus.Disconnected)
    val status: StateFlow<TunnelStatus> = _status.asStateFlow()

    private val _events = MutableSharedFlow<TunnelStatus>(replay = 1)
    val events: SharedFlow<TunnelStatus> = _events.asSharedFlow()

    suspend fun publish(status: TunnelStatus) {
        _status.value = status
        _events.emit(status)
    }
}

