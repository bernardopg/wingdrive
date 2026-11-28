package com.wingdrive.core

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.net.wifi.WifiManager
import android.os.Build
import android.os.IBinder
import android.os.PowerManager
import android.util.Log
import androidx.core.app.NotificationCompat
import androidx.core.app.ServiceCompat
import androidx.core.content.ContextCompat

/**
 * Keeps the embedded core running while the app is in the background (MOB-018).
 *
 * Android freezes or kills a backgrounded app within minutes and cuts its network in Doze, so
 * sync with paired devices and incoming transfers stopped as soon as the user left the app. A
 * foreground service raises the process to foreground priority, which keeps network access and
 * honours wake locks in Doze. The partial wake lock keeps the CPU awake for the core's timers
 * and connection keepalives with the screen off. The service type is `connectedDevice`: the
 * core talks to the user's own paired devices, and the multicast permission the app already
 * holds satisfies the type's prerequisite on Android 14+.
 *
 * When Android restarts the service after killing the process, no JavaScript runs, so the
 * service starts the core itself with the data folder and device name the app last used.
 */
class WingCoreService : Service() {
    private var wakeLock: PowerManager.WakeLock? = null
    private var multicastLock: WifiManager.MulticastLock? = null

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onCreate() {
        super.onCreate()
        val notification = buildNotification(this)
        try {
            ServiceCompat.startForeground(this, NOTIFICATION_ID, notification, foregroundType())
        } catch (e: Exception) {
            // Android 12+ refuses a foreground service started from the background; the app
            // starts it again the next time it opens.
            Log.w(TAG, "Could not enter the foreground: ${e.message}")
            stopSelf()
            return
        }
        running = true
        acquireLocks()
        ensureCoreRunning()
        Log.i(TAG, "Background service started")
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (!BackgroundSyncPrefs.isEnabled(this)) {
            stopSelf()
            return START_NOT_STICKY
        }
        return START_STICKY
    }

    override fun onDestroy() {
        running = false
        releaseLocks()
        Log.i(TAG, "Background service stopped")
        super.onDestroy()
    }

    private fun ensureCoreRunning() {
        val dataDir = BackgroundSyncPrefs.dataDir(this) ?: return
        try {
            // The core ignores a second initialization, so this is safe when the app already
            // started it.
            val result = WingMobileCoreModule.initializeCore(dataDir, BackgroundSyncPrefs.deviceName(this))
            if (result != 0) Log.w(TAG, "Core did not start from the service: $result")
        } catch (e: Throwable) {
            Log.e(TAG, "Failed to start the core from the service: ${e.message}")
        }
    }

    private fun acquireLocks() {
        try {
            val power = getSystemService(Context.POWER_SERVICE) as PowerManager
            wakeLock = power.newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, "WingDrive:BackgroundSync").apply {
                setReferenceCounted(false)
                acquire()
            }
        } catch (e: Exception) {
            Log.w(TAG, "Failed to acquire the wake lock: ${e.message}")
        }
        try {
            val wifi = applicationContext.getSystemService(Context.WIFI_SERVICE) as? WifiManager
            multicastLock = wifi?.createMulticastLock("WingDriveBackgroundDiscovery")?.apply {
                setReferenceCounted(false)
                acquire()
            }
        } catch (e: Exception) {
            // Paired devices are still reached through the relay without local discovery.
            Log.w(TAG, "Failed to acquire the multicast lock: ${e.message}")
        }
    }

    private fun releaseLocks() {
        try {
            wakeLock?.takeIf { it.isHeld }?.release()
        } catch (e: Exception) {
            Log.w(TAG, "Failed to release the wake lock: ${e.message}")
        }
        wakeLock = null
        try {
            multicastLock?.takeIf { it.isHeld }?.release()
        } catch (e: Exception) {
            Log.w(TAG, "Failed to release the multicast lock: ${e.message}")
        }
        multicastLock = null
    }

    companion object {
        private const val TAG = "WingMobileCore"
        private const val CHANNEL_ID = "wingdrive_background"
        private const val NOTIFICATION_ID = 6969

        @Volatile
        var running = false
            private set

        /** Starts the service; only works while the app is in the foreground. */
        fun start(context: Context): Boolean {
            return try {
                ContextCompat.startForegroundService(context, Intent(context, WingCoreService::class.java))
                true
            } catch (e: Exception) {
                Log.w(TAG, "Could not start the background service: ${e.message}")
                false
            }
        }

        fun stop(context: Context) {
            context.stopService(Intent(context, WingCoreService::class.java))
        }

        private fun foregroundType(): Int =
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                ServiceInfo.FOREGROUND_SERVICE_TYPE_CONNECTED_DEVICE
            } else {
                0
            }

        private fun buildNotification(context: Context): Notification {
            val manager = context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                // Low importance: no sound and no heads-up for a notice that stays all day.
                val channel = NotificationChannel(
                    CHANNEL_ID,
                    context.getString(R.string.wingdrive_background_channel),
                    NotificationManager.IMPORTANCE_LOW,
                ).apply {
                    description = context.getString(R.string.wingdrive_background_channel_description)
                    setShowBadge(false)
                }
                manager.createNotificationChannel(channel)
            }
            val launch = context.packageManager.getLaunchIntentForPackage(context.packageName)
            val contentIntent = launch?.let {
                PendingIntent.getActivity(
                    context,
                    0,
                    it,
                    PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
                )
            }
            return NotificationCompat.Builder(context, CHANNEL_ID)
                .setSmallIcon(R.drawable.wingdrive_notification)
                .setContentTitle(context.getString(R.string.wingdrive_background_title))
                .setContentText(context.getString(R.string.wingdrive_background_text))
                .setContentIntent(contentIntent)
                .setOngoing(true)
                .setSilent(true)
                .setPriority(NotificationCompat.PRIORITY_LOW)
                .setCategory(NotificationCompat.CATEGORY_SERVICE)
                .setForegroundServiceBehavior(NotificationCompat.FOREGROUND_SERVICE_IMMEDIATE)
                .build()
        }
    }
}

/** The background choice and what the service needs to start the core without the app. */
object BackgroundSyncPrefs {
    private const val FILE = "wingdrive_background"
    private const val KEY_ENABLED = "enabled"
    private const val KEY_DATA_DIR = "data_dir"
    private const val KEY_DEVICE_NAME = "device_name"

    private fun prefs(context: Context) =
        context.applicationContext.getSharedPreferences(FILE, Context.MODE_PRIVATE)

    fun isEnabled(context: Context) = prefs(context).getBoolean(KEY_ENABLED, false)

    fun setEnabled(context: Context, enabled: Boolean) {
        prefs(context).edit().putBoolean(KEY_ENABLED, enabled).apply()
    }

    fun dataDir(context: Context): String? = prefs(context).getString(KEY_DATA_DIR, null)

    fun deviceName(context: Context): String? = prefs(context).getString(KEY_DEVICE_NAME, null)

    fun rememberCoreArguments(context: Context, dataDir: String, deviceName: String?) {
        prefs(context).edit()
            .putString(KEY_DATA_DIR, dataDir)
            .putString(KEY_DEVICE_NAME, deviceName)
            .apply()
    }
}
