package dev.cryprq.logging

import android.content.Context
import android.net.Uri
import android.os.Environment
import android.util.Log
import androidx.core.content.FileProvider
import dev.cryprq.settings.CrypRqSettings
import kotlinx.coroutines.flow.StateFlow
import java.io.File
import java.io.FileWriter
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

object CrypRqLogger {

    private const val TAG = "CrypRqLogger"
    private const val MAX_LOG_BYTES = 256 * 1024 // 256 KiB before rotatation
    private const val LOG_FILE_NAME = "cryprq.log"
    private const val LOG_FILE_BACKUP = "cryprq.log.1"

    private lateinit var appContext: Context
    private val timestampFormatter = SimpleDateFormat("yyyy-MM-dd HH:mm:ss.SSS", Locale.US)

    fun init(context: Context) {
        if (::appContext.isInitialized) return
        appContext = context.applicationContext
        CrypRqSettings.init(appContext)
    }

    fun loggingEnabled(): StateFlow<Boolean> = CrypRqSettings.loggingEnabled()

    fun setLoggingEnabled(enabled: Boolean) {
        CrypRqSettings.setLoggingEnabled(enabled)
        if (enabled) {
            Log.i(TAG, "On-device logging enabled")
            appendLine("Logging enabled")
        } else {
            appendLine("Logging disabled")
            Log.i(TAG, "On-device logging disabled")
        }
    }

    fun log(message: String) {
        if (!loggingEnabled().value) return
        appendLine(message)
        Log.i(TAG, message)
    }

    fun shareLogsIntent(): android.content.Intent? {
        if (!::appContext.isInitialized) return null
        val logFile = logFile()
        if (!logFile.exists() || logFile.length() == 0L) {
            return null
        }
        val authority = "${appContext.packageName}.fileprovider"
        val uri: Uri = FileProvider.getUriForFile(appContext, authority, logFile)
        return android.content.Intent(android.content.Intent.ACTION_SEND).apply {
            type = "text/plain"
            putExtra(android.content.Intent.EXTRA_STREAM, uri)
            addFlags(android.content.Intent.FLAG_GRANT_READ_URI_PERMISSION)
        }
    }

    fun purge() {
        if (!::appContext.isInitialized) return
        logFile().delete()
        backupLogFile().delete()
    }

    private fun appendLine(message: String) {
        if (!::appContext.isInitialized) return
        try {
            rotateIfNeeded()
            val file = logFile()
            file.parentFile?.mkdirs()
            FileWriter(file, true).use { writer ->
                val timestamp = timestampFormatter.format(Date())
                writer.appendLine("$timestamp | $message")
            }
        } catch (t: Throwable) {
            Log.w(TAG, "Failed to write log", t)
        }
    }

    private fun rotateIfNeeded() {
        val file = logFile()
        if (!file.exists()) return
        if (file.length() < MAX_LOG_BYTES) return
        val backup = backupLogFile()
        if (backup.exists()) {
            backup.delete()
        }
        file.renameTo(backup)
        file.delete()
    }

    private fun logFile(): File {
        val logsDir = appContext.getExternalFilesDir(Environment.DIRECTORY_DOCUMENTS)
            ?.resolve("CrypRQ/logs")
            ?: appContext.filesDir.resolve("logs")
        return logsDir.resolve(LOG_FILE_NAME)
    }

    private fun backupLogFile(): File {
        val logsDir = appContext.getExternalFilesDir(Environment.DIRECTORY_DOCUMENTS)
            ?.resolve("CrypRQ/logs")
            ?: appContext.filesDir.resolve("logs")
        return logsDir.resolve(LOG_FILE_BACKUP)
    }
}

