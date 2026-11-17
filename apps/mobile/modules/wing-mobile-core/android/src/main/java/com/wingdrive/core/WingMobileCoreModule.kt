package com.wingdrive.core

import expo.modules.kotlin.modules.Module
import expo.modules.kotlin.modules.ModuleDefinition
import expo.modules.kotlin.Promise

class WingMobileCoreModule : Module() {
    private var listeners = 0
    private var logListeners = 0
    private var registeredWithRust = false
    private var logRegisteredWithRust = false

    // Android drops multicast packets for apps that hold no multicast lock, so the core's mDNS
    // discovery never saw other devices and logged "error sending mDNS: Operation not
    // permitted" every few seconds (MOB-017). Held while the core runs.
    private var multicastLock: android.net.wifi.WifiManager.MulticastLock? = null

    init {
        try {
            System.loadLibrary("wing_mobile_core")
        } catch (e: UnsatisfiedLinkError) {
            android.util.Log.e("WingMobileCore", "Failed to load native library: ${e.message}")
        }
    }

    override fun definition() = ModuleDefinition {
        Name("WingMobileCore")

        Events("WingCoreEvent", "WingCoreLog")

        OnStartObserving("WingCoreEvent") {
            android.util.Log.i("WingMobileCore", "📡 OnStartObserving WingCoreEvent triggered")

            if (!registeredWithRust) {
                try {
                    android.util.Log.i("WingMobileCore", "🚀 Registering event listener...")
                    registerCoreEventListener()
                    registeredWithRust = true
                    android.util.Log.i("WingMobileCore", "✅ Event listener registered with Rust")
                } catch (e: Exception) {
                    android.util.Log.e("WingMobileCore", "Failed to register event listener: ${e.message}")
                }
            }

            listeners++
            android.util.Log.i("WingMobileCore", "📊 WingCoreEvent listeners: $listeners")
        }

        OnStopObserving("WingCoreEvent") {
            listeners--
            android.util.Log.i("WingMobileCore", "📉 WingCoreEvent listeners: $listeners")
        }

        OnStartObserving("WingCoreLog") {
            android.util.Log.i("WingMobileCore", "📡 OnStartObserving WingCoreLog triggered")

            if (!logRegisteredWithRust) {
                try {
                    android.util.Log.i("WingMobileCore", "🚀 Registering log listener...")
                    registerCoreLogListener()
                    logRegisteredWithRust = true
                    android.util.Log.i("WingMobileCore", "✅ Log listener registered with Rust")
                } catch (e: Exception) {
                    android.util.Log.e("WingMobileCore", "Failed to register log listener: ${e.message}")
                }
            }

            logListeners++
            android.util.Log.i("WingMobileCore", "📊 WingCoreLog listeners: $logListeners")
        }

        OnStopObserving("WingCoreLog") {
            logListeners--
            android.util.Log.i("WingMobileCore", "📉 WingCoreLog listeners: $logListeners")
        }

        Function("initialize") { dataDir: String?, deviceName: String? ->
            val dir = dataDir ?: appContext.persistentFilesDirectory?.absolutePath
                ?: throw Exception("No data directory available")

            acquireMulticastLock()
            try {
                initializeCore(dir, deviceName)
            } catch (e: Exception) {
                android.util.Log.e("WingMobileCore", "Failed to initialize core: ${e.message}")
                -1
            }
        }

        AsyncFunction("sendMessage") { query: String, promise: Promise ->
            try {
                handleCoreMsg(query, WingCorePromise(promise))
            } catch (e: Exception) {
                promise.reject("CORE_ERROR", e.message ?: "Unknown error", e)
            }
        }

        Function("shutdown") {
            try {
                shutdownCore()
            } catch (e: Exception) {
                android.util.Log.e("WingMobileCore", "Failed to shutdown core: ${e.message}")
            }
            releaseMulticastLock()
        }

        OnDestroy {
            releaseMulticastLock()
        }

        // A file manager needs to read shared storage: "All files access" on Android 11+,
        // the storage permission before (MOB-017).
        Function("hasAllFilesAccess") {
            val context = appContext.reactContext ?: return@Function false
            if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.R) {
                android.os.Environment.isExternalStorageManager()
            } else {
                context.checkSelfPermission(android.Manifest.permission.READ_EXTERNAL_STORAGE) ==
                    android.content.pm.PackageManager.PERMISSION_GRANTED
            }
        }

        // Opens the system page where the user grants it; the app re-checks when it returns.
        Function("openAllFilesAccessSettings") {
            val context = appContext.reactContext ?: return@Function false
            val packageUri = android.net.Uri.parse("package:${context.packageName}")
            val intent = if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.R) {
                android.content.Intent(
                    android.provider.Settings.ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION,
                    packageUri,
                )
            } else {
                android.content.Intent(
                    android.provider.Settings.ACTION_APPLICATION_DETAILS_SETTINGS,
                    packageUri,
                )
            }
            intent.addFlags(android.content.Intent.FLAG_ACTIVITY_NEW_TASK)
            try {
                context.startActivity(intent)
                true
            } catch (e: Exception) {
                android.util.Log.e("WingMobileCore", "Failed to open storage settings: ${e.message}")
                false
            }
        }
    }

    private fun acquireMulticastLock() {
        if (multicastLock?.isHeld == true) return
        val context = appContext.reactContext?.applicationContext ?: return
        try {
            val service = context.getSystemService(android.content.Context.WIFI_SERVICE)
            val wifi = service as? android.net.wifi.WifiManager ?: return
            multicastLock = wifi.createMulticastLock("WingDriveDiscovery").apply {
                // One release undoes any number of acquires.
                setReferenceCounted(false)
                acquire()
            }
            android.util.Log.i("WingMobileCore", "Multicast lock held for local discovery")
        } catch (e: Exception) {
            // Without the lock, devices are still found through the relay and by pairing code.
            android.util.Log.w("WingMobileCore", "Failed to acquire multicast lock: ${e.message}")
        }
    }

    private fun releaseMulticastLock() {
        try {
            multicastLock?.takeIf { it.isHeld }?.release()
        } catch (e: Exception) {
            android.util.Log.w("WingMobileCore", "Failed to release multicast lock: ${e.message}")
        }
        multicastLock = null
    }

    fun getDataDirectory(): String {
        return appContext.persistentFilesDirectory?.absolutePath ?: ""
    }

    fun sendCoreEvent(body: String) {
        if (listeners > 0) {
            this@WingMobileCoreModule.sendEvent("WingCoreEvent", mapOf("body" to body))
        }
    }

    fun sendCoreLog(body: String) {
        if (logListeners > 0) {
            this@WingMobileCoreModule.sendEvent("WingCoreLog", mapOf("body" to body))
        }
    }

    // Native methods - will throw UnsatisfiedLinkError if library not loaded
    private external fun registerCoreEventListener()
    private external fun registerCoreLogListener()
    private external fun initializeCore(dataDir: String, deviceName: String?): Int
    private external fun handleCoreMsg(query: String, promise: WingCorePromise)
    private external fun shutdownCore()
}

class WingCorePromise(private val promise: Promise) {
    fun resolve(msg: String) {
        promise.resolve(msg)
    }

    fun reject(error: String) {
        promise.reject("CORE_ERROR", error, null)
    }
}
