package dev.cryprq.app

import android.content.Context
import android.content.pm.PackageManager
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import dev.cryprq.tunnel.CrypRqVpnService
import org.junit.Assert.assertNotNull
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class CrypRqInstrumentationTest {

    @Test
    fun vpnServiceRegistered() {
        val context = ApplicationProvider.getApplicationContext<Context>()
        val component = android.content.ComponentName(context, CrypRqVpnService::class.java)
        val info = context.packageManager.getServiceInfo(
            component,
            PackageManager.GET_META_DATA
        )
        assertNotNull(info)
    }
}

