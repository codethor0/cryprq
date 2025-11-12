package dev.cryprq.settings

import android.content.Context
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

object CrypRqSettings {

    private const val PREFS_NAME = "cryprq_prefs"
    private const val KEY_LOGGING_ENABLED = "logging_enabled"
    private const val KEY_METRICS_ENABLED = "metrics_enabled"
    private const val KEY_ROTATION_MINUTES = "rotation_minutes"

    private const val DEFAULT_ROTATION_MINUTES = 5
    private const val MIN_ROTATION_MINUTES = 1
    private const val MAX_ROTATION_MINUTES = 60

    private lateinit var appContext: Context
    private val loggingEnabledState = MutableStateFlow(false)
    private val metricsEnabledState = MutableStateFlow(false)
    private val rotationMinutesState = MutableStateFlow(DEFAULT_ROTATION_MINUTES)

    fun init(context: Context) {
        if (::appContext.isInitialized) return
        appContext = context.applicationContext
        val prefs = prefs()
        loggingEnabledState.value = prefs.getBoolean(KEY_LOGGING_ENABLED, false)
        metricsEnabledState.value = prefs.getBoolean(KEY_METRICS_ENABLED, false)
        rotationMinutesState.value = prefs.getInt(KEY_ROTATION_MINUTES, DEFAULT_ROTATION_MINUTES)
    }

    fun loggingEnabled(): StateFlow<Boolean> = loggingEnabledState
    fun metricsEnabled(): StateFlow<Boolean> = metricsEnabledState
    fun rotationMinutes(): StateFlow<Int> = rotationMinutesState

    fun setLoggingEnabled(enabled: Boolean) {
        ensureInit()
        prefs().edit().putBoolean(KEY_LOGGING_ENABLED, enabled).apply()
        loggingEnabledState.value = enabled
    }

    fun setMetricsEnabled(enabled: Boolean) {
        ensureInit()
        prefs().edit().putBoolean(KEY_METRICS_ENABLED, enabled).apply()
        metricsEnabledState.value = enabled
    }

    fun setRotationMinutes(minutes: Int) {
        ensureInit()
        val clamped = minutes.coerceAtLeast(MIN_ROTATION_MINUTES).coerceAtMost(MAX_ROTATION_MINUTES)
        prefs().edit().putInt(KEY_ROTATION_MINUTES, clamped).apply()
        rotationMinutesState.value = clamped
    }

    private fun prefs() = appContext.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)

    private fun ensureInit() {
        check(::appContext.isInitialized) { "CrypRqSettings.init must be called before use" }
    }
}

