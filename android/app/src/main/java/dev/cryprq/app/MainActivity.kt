package dev.cryprq.app

import android.content.Intent
import android.content.res.ColorStateList
import android.net.VpnService
import android.os.Bundle
import android.view.View
import android.view.Menu
import android.view.MenuItem
import androidx.activity.ComponentActivity
import androidx.activity.viewModels
import androidx.core.content.ContextCompat
import androidx.core.view.isVisible
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import dev.cryprq.app.databinding.ActivityMainBinding
import dev.cryprq.logging.CrypRqLogger
import dev.cryprq.tunnel.CrypRqTunnelController
import dev.cryprq.tunnel.TunnelEvent
import dev.cryprq.tunnel.TunnelStatus
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import java.util.TimeZone

class MainActivity : ComponentActivity() {

    private lateinit var binding: ActivityMainBinding
    private val controller: CrypRqTunnelController by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        CrypRqLogger.init(applicationContext)
        binding = ActivityMainBinding.inflate(layoutInflater)
        setContentView(binding.root)

        binding.startButton.setOnClickListener { startTunnel() }
        binding.stopButton.setOnClickListener { controller.stopTunnel(this) }

        lifecycleScope.launch {
            repeatOnLifecycle(Lifecycle.State.STARTED) {
                launch {
                    controller.status.collectLatest { status ->
                        renderStatus(status)
                    }
                }
                launch {
                    controller.events.collectLatest { events ->
                        renderEvents(events)
                    }
                }
            }
        }
    }

    override fun onCreateOptionsMenu(menu: Menu): Boolean {
        menuInflater.inflate(R.menu.menu_main, menu)
        return true
    }

    override fun onOptionsItemSelected(item: MenuItem): Boolean {
        return when (item.itemId) {
            R.id.action_settings -> {
                startActivity(Intent(this, SettingsActivity::class.java))
                true
            }
            else -> super.onOptionsItemSelected(item)
        }
    }

    private fun renderStatus(status: TunnelStatus) {
        binding.statusChip.text = getString(status.labelRes)
        binding.statusChip.chipIcon = ContextCompat.getDrawable(this, statusIcon(status))
        binding.statusChip.chipBackgroundColor = ContextCompat.getColorStateList(this, statusColor(status))
        val textColor = ContextCompat.getColor(this, statusTextColor(status))
        binding.statusChip.setTextColor(textColor)
        binding.statusChip.chipIconTint = ColorStateList.valueOf(textColor)
        binding.connectionSpinner.isVisible = status == TunnelStatus.Connecting
        binding.startButton.isEnabled = status == TunnelStatus.Disconnected
        binding.stopButton.isEnabled = status != TunnelStatus.Disconnected
    }

    private fun renderEvents(events: List<TunnelEvent>) {
        if (events.isEmpty()) {
            binding.logContent.text = getString(R.string.log_empty_state)
            return
        }

        val builder = StringBuilder()
        events.forEach { event ->
            builder.append(timeFormatter.format(Date(event.timestamp)))
            builder.append("  •  ")
            builder.append(
                when (event) {
                    is TunnelEvent.StatusChanged -> getString(event.status.labelRes)
                    is TunnelEvent.Message -> event.text
                }
            )
            builder.appendLine()
        }
        binding.logContent.text = builder.toString().trim()
        binding.logScroll.post {
            binding.logScroll.fullScroll(View.FOCUS_DOWN)
        }
    }

    private fun startTunnel() {
        val prepareIntent = VpnService.prepare(this)
        if (prepareIntent != null) {
            startActivityForResult(prepareIntent, REQUEST_PREPARE)
        } else {
            controller.startTunnel(this)
        }
    }

    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        if (requestCode == REQUEST_PREPARE && resultCode == RESULT_OK) {
            controller.startTunnel(this)
        }
    }

    private fun statusIcon(status: TunnelStatus): Int = when (status) {
        TunnelStatus.Connected -> R.drawable.ic_status_connected
        TunnelStatus.Connecting -> R.drawable.ic_status_connecting
        TunnelStatus.Disconnected -> R.drawable.ic_status_disconnected
    }

    private fun statusColor(status: TunnelStatus): Int = when (status) {
        TunnelStatus.Connected -> R.color.status_connected
        TunnelStatus.Connecting -> R.color.status_connecting
        TunnelStatus.Disconnected -> R.color.status_disconnected
    }

    private fun statusTextColor(status: TunnelStatus): Int = when (status) {
        TunnelStatus.Connecting -> android.R.color.black
        TunnelStatus.Connected,
        TunnelStatus.Disconnected -> android.R.color.white
    }

    companion object {
        private const val REQUEST_PREPARE = 100
        private val timeFormatter = SimpleDateFormat("HH:mm:ss", Locale.getDefault()).apply {
            timeZone = TimeZone.getDefault()
        }
    }
}

