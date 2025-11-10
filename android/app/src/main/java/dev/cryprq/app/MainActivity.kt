package dev.cryprq.app

import android.content.Intent
import android.net.VpnService
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.viewModels
import androidx.lifecycle.lifecycleScope
import dev.cryprq.app.databinding.ActivityMainBinding
import dev.cryprq.tunnel.CrypRqTunnelController
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch

class MainActivity : ComponentActivity() {

    private lateinit var binding: ActivityMainBinding
    private val controller: CrypRqTunnelController by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        binding = ActivityMainBinding.inflate(layoutInflater)
        setContentView(binding.root)

        binding.startButton.setOnClickListener { startTunnel() }
        binding.stopButton.setOnClickListener { controller.stopTunnel(this) }

        lifecycleScope.launch {
            controller.status.collectLatest { status ->
                binding.statusLabel.text = getString(status.labelRes)
            }
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

    companion object {
        private const val REQUEST_PREPARE = 100
    }
}

