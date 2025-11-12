package dev.cryprq.app

import android.os.Bundle
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.core.view.isVisible
import androidx.lifecycle.lifecycleScope
import com.google.android.material.dialog.MaterialAlertDialogBuilder
import dev.cryprq.app.databinding.ActivitySettingsBinding
import dev.cryprq.logging.CrypRqLogger
import dev.cryprq.settings.CrypRqSettings
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch

class SettingsActivity : ComponentActivity() {

    private lateinit var binding: ActivitySettingsBinding

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        CrypRqLogger.init(applicationContext)
        binding = ActivitySettingsBinding.inflate(layoutInflater)
        setContentView(binding.root)

        binding.toolbar.setNavigationOnClickListener { finish() }

        lifecycleScope.launch {
            CrypRqLogger.loggingEnabled().collectLatest { enabled ->
                if (binding.loggingSwitch.isChecked != enabled) {
                    binding.loggingSwitch.isChecked = enabled
                }
                binding.shareLogsButton.isEnabled = enabled
                binding.logsInfoText.isVisible = enabled
            }
        }

        lifecycleScope.launch {
            CrypRqSettings.metricsEnabled().collectLatest { enabled ->
                if (binding.metricsSwitch.isChecked != enabled) {
                    binding.metricsSwitch.isChecked = enabled
                }
            }
        }

        lifecycleScope.launch {
            CrypRqSettings.rotationMinutes().collectLatest { minutes ->
                if (binding.rotationSlider.value.toInt() != minutes) {
                    binding.rotationSlider.value = minutes.toFloat()
                }
                binding.rotationValue.text = getString(R.string.settings_rotation_value, minutes)
            }
        }

        binding.loggingSwitch.setOnCheckedChangeListener { _, isChecked ->
            CrypRqLogger.setLoggingEnabled(isChecked)
        }

        binding.metricsSwitch.setOnCheckedChangeListener { _, isChecked ->
            CrypRqSettings.setMetricsEnabled(isChecked)
        }

        binding.rotationSlider.addOnChangeListener { _, value, fromUser ->
            if (fromUser) {
                CrypRqSettings.setRotationMinutes(value.toInt())
            }
        }

        binding.peerConfigButton.setOnClickListener {
            MaterialAlertDialogBuilder(this)
                .setTitle(R.string.settings_peer_config_title)
                .setMessage(getString(R.string.settings_peer_config_stub))
                .setPositiveButton(android.R.string.ok, null)
                .show()
        }

        binding.shareLogsButton.setOnClickListener {
            val intent = CrypRqLogger.shareLogsIntent()
            if (intent == null) {
                Toast.makeText(this, R.string.settings_share_logs_empty, Toast.LENGTH_SHORT).show()
            } else {
                startActivity(
                    android.content.Intent.createChooser(
                        intent,
                        getString(R.string.settings_share_logs_title)
                    )
                )
            }
        }

        binding.clearLogsButton.setOnClickListener {
            CrypRqLogger.purge()
            Toast.makeText(this, R.string.settings_logs_cleared, Toast.LENGTH_SHORT).show()
        }
    }
}

